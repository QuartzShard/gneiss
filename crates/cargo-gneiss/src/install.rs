//! `cargo gneiss install`: send an app bundle to a watch, through the phone app's developer
//! connection or an emulator that pebble-tool is running.
//!
//! Both speak the developer connection's WebSocket protocol (libpebble2's websocket transport):
//! each binary message is an endpoint byte, then its payload. The phone installs a bundle itself;
//! Pebble Protocol packets to and from the watch ride inside relay messages.
use std::{
	net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream, ToSocketAddrs},
	path::PathBuf,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	time::{Duration, Instant},
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail, eyre},
};
use gneiss_types::Platform;
use tungstenite::{Message, WebSocket};

mod adb;

/// Developer connection endpoints
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Endpoint {
	RelayFromWatch = 0x00,
	RelayToWatch   = 0x01,
	PhoneAppLog    = 0x02,
	InstallBundle  = 0x04,
	InstallStatus  = 0x05,
}

impl Endpoint {
	const ALL: [Self; 5] = [
		Self::RelayFromWatch,
		Self::RelayToWatch,
		Self::PhoneAppLog,
		Self::InstallBundle,
		Self::InstallStatus,
	];

	fn from_raw(byte: u8) -> Option<Self> {
		Self::ALL
			.into_iter()
			.find(|endpoint| *endpoint as u8 == byte)
	}

	/// A message's endpoint and payload.
	fn split(message: &[u8]) -> Option<(Self, &[u8])> {
		let (&byte, payload) = message.split_first()?;
		Some((Self::from_raw(byte)?, payload))
	}
}

/// Pebble Protocol endpoint for app logs: the shipping switch towards the watch, log lines back
const APP_LOGS: u16 = 2006;
const PHONE_PORT: u16 = 9000;
/// As long as pebble-tool gives the phone to transfer and install a bundle
const INSTALL_TIMEOUT: Duration = Duration::from_secs(300);
/// How often a blocked read wakes up to check for a deadline or Ctrl-C
const POLL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, clap::Parser)]
/// Install an app bundle on a watch
pub struct Install {
	/// The .pbw to install. Defaults to this project's `cargo gneiss build` output
	pbw:    Option<PathBuf>,
	#[command(flatten)]
	target: Target,
	/// Stay connected and print the watch's app logs until interrupted
	#[arg(long)]
	logs:   bool,
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = false)]
struct Target {
	/// The phone's address, `host` or `host:port`, with the Pebble app's developer connection on.
	/// An IPv6 address goes bare or bracketed, `[addr]:port` to give a port. A link-local one
	/// names its interface, as `fe80::1%wlan0`
	#[arg(long)]
	phone:    Option<String>,
	/// The platform of an emulator that pebble-tool already has running
	#[arg(long)]
	emulator: Option<Platform>,
	/// The Pebble app on an Android phone over USB, through the adb server. Alone, the one phone
	/// attached; `--adb=DEVICE` picks one as `adb devices` lists it
	#[arg(long, value_name = "DEVICE", num_args = 0..=1, require_equals = true, default_missing_value = "")]
	adb:      Option<String>,
}

impl Target {
	fn connect(&self) -> Result<WebSocket<TcpStream>> {
		match (&self.phone, self.emulator, &self.adb) {
			(Some(phone), ..) => connect(&phone_host(phone)?),
			(_, Some(platform), _) => connect(&format!("127.0.0.1:{}", emulator_port(platform)?)),
			(.., Some(device)) => {
				let (stream, serial) = adb::open(Some(device.as_str()).filter(|d| !d.is_empty()))?;
				handshake(stream, &format!("localhost:{PHONE_PORT}"), &serial)
			}
			_ => bail!("Pass --phone, --emulator or --adb"),
		}
	}
}

impl Install {
	pub fn install(&self) -> Result<()> {
		let pbw_path = match &self.pbw {
			Some(path) => path.clone(),
			None => default_pbw()?,
		};
		let pbw = std::fs::read(&pbw_path)
			.wrap_err_with(|| format!("Can't read {}", pbw_path.display()))?;
		let mut ws = self.target.connect()?;

		let stop = Arc::new(AtomicBool::new(false));
		if self.logs {
			let stop = Arc::clone(&stop);
			ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed))?;
			set_log_shipping(&mut ws, true)?;
		}

		log::info!("Installing {}...", pbw_path.display());
		ws.send(Message::binary(frame(Endpoint::InstallBundle, &pbw)))?;
		let deadline = Instant::now() + INSTALL_TIMEOUT;
		loop {
			if Instant::now() > deadline {
				bail!("Timed out waiting for the install to finish");
			}
			let Some(data) = read(&mut ws)? else { continue };
			if let Some((Endpoint::InstallStatus, status)) = Endpoint::split(&data) {
				match status.first_chunk::<4>().map(|s| u32::from_be_bytes(*s)) {
					Some(0) => break,
					_ => bail!("The app install failed"),
				}
			}
			handle(&data, self.logs);
		}
		log::info!("App install succeeded");

		if self.logs {
			log::info!("Printing app logs, Ctrl-C to stop");
			while !stop.load(Ordering::Relaxed) {
				match read(&mut ws) {
					Ok(Some(data)) => handle(&data, true),
					Ok(None) => {}
					Err(e) => {
						log::warn!("{e}");
						return Ok(());
					}
				}
			}
			set_log_shipping(&mut ws, false)?;
		}
		Ok(())
	}
}

/// Where `cargo gneiss build` puts this project's bundle.
fn default_pbw() -> Result<PathBuf> {
	let (meta, pkg, _) = gneiss_build::read_pbl_metadata()?;
	let path = meta
		.target_directory
		.join(Platform::default().app_target_triple())
		.join("release")
		.join(format!("{}.pbw", pkg.name));
	if !path.exists() {
		bail!("No bundle at {path}: run `cargo gneiss build` first, or name a .pbw");
	}
	Ok(path.into())
}

/// The pypkjs port of the one running emulator for `platform`, from the state file pebble-tool
/// keeps in the temp directory.
fn emulator_port(platform: Platform) -> Result<u16> {
	let path = std::env::temp_dir().join("pb-emulator.json");
	let no_emulator = || {
		eyre!(
			"No {platform} emulator is running. Start one with pebble-tool, e.g. `pebble install \
			 --emulator {platform}`"
		)
	};
	let info: serde_json::Value =
		serde_json::from_slice(&std::fs::read(&path).map_err(|_| no_emulator())?)?;
	let versions = info
		.get(platform.as_str())
		.and_then(serde_json::Value::as_object)
		.ok_or_else(no_emulator)?;
	// An entry outlives its emulator, so only count one whose pypkjs still answers
	let alive: Vec<(&String, u16)> = versions
		.iter()
		.filter_map(|(version, entry)| {
			let port = u16::try_from(entry.get("pypkjs")?.get("port")?.as_u64()?).ok()?;
			let addr = SocketAddr::from(([127, 0, 0, 1], port));
			TcpStream::connect_timeout(&addr, Duration::from_millis(500)).ok()?;
			Some((version, port))
		})
		.collect();
	match alive.as_slice() {
		[] => Err(no_emulator()),
		[(_, port)] => Ok(*port),
		_ => bail!(
			"Several {platform} emulators are running (SDK {}); stop all but one",
			alive
				.iter()
				.map(|(version, _)| version.as_str())
				.collect::<Vec<_>>()
				.join(", ")
		),
	}
}

/// `phone` as `host:port`, with [`PHONE_PORT`] if it has none. An IPv6 zone, `%wlan0` or `%3`,
/// becomes its interface index.
fn phone_host(phone: &str) -> Result<String> {
	let (addr, port) = match phone
		.strip_prefix('[')
		.and_then(|rest| rest.split_once(']'))
	{
		Some((addr, "")) => (addr, PHONE_PORT),
		Some((addr, port)) => {
			let port = port
				.strip_prefix(':')
				.and_then(|port| port.parse().ok())
				.ok_or_eyre(format!("{phone}: expected [address]:port"))?;
			(addr, port)
		}
		None if phone.matches(':').count() > 1 => (phone, PHONE_PORT),
		None if phone.contains(':') => return Ok(phone.to_owned()),
		None => return Ok(format!("{phone}:{PHONE_PORT}")),
	};
	let (ip, zone) = addr
		.split_once('%')
		.map_or((addr, None), |(ip, zone)| (ip, Some(zone)));
	let ip: Ipv6Addr = ip
		.parse()
		.wrap_err_with(|| format!("{ip} isn't an IPv6 address"))?;
	let scope = match zone {
		None => 0,
		Some(zone) => match zone.parse() {
			Ok(index) => index,
			Err(_) => interface_index(zone)?,
		},
	};
	if ip.is_unicast_link_local() && scope == 0 {
		bail!("{ip} is link-local, so it needs the interface to reach it on, such as {ip}%wlan0");
	}
	Ok(SocketAddrV6::new(ip, port, 0, scope).to_string())
}

#[cfg(unix)]
fn interface_index(name: &str) -> Result<u32> {
	let c_name = std::ffi::CString::new(name)?;
	// SAFETY: a NUL-terminated string
	match unsafe { libc::if_nametoindex(c_name.as_ptr()) } {
		0 => bail!("No network interface {name}"),
		index => Ok(index),
	}
}

#[cfg(windows)]
fn interface_index(name: &str) -> Result<u32> {
	let c_name = std::ffi::CString::new(name)?;
	// SAFETY: a NUL-terminated string
	match unsafe {
		windows_sys::Win32::NetworkManagement::IpHelper::if_nametoindex(c_name.as_ptr().cast())
	} {
		0 => bail!("No network interface {name}"),
		index => Ok(index),
	}
}

fn connect(host: &str) -> Result<WebSocket<TcpStream>> {
	let addr = host
		.to_socket_addrs()
		.wrap_err_with(|| format!("Can't resolve {host}"))?
		.next()
		.ok_or_eyre(format!("Can't resolve {host}"))?;
	let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(5)).wrap_err_with(|| {
		format!("Can't reach {host}: is the Pebble app's developer connection on?")
	})?;
	let url_host = match addr {
		SocketAddr::V6(v6) if v6.scope_id() != 0 => {
			SocketAddrV6::new(*v6.ip(), v6.port(), 0, 0).to_string()
		}
		_ => host.to_owned(),
	};
	handshake(stream, &url_host, host)
}

/// A WebSocket over `stream`, to `url_host`; `peer` names it in errors.
fn handshake(stream: TcpStream, url_host: &str, peer: &str) -> Result<WebSocket<TcpStream>> {
	let (mut ws, _) = tungstenite::client(format!("ws://{url_host}/"), stream)
		.map_err(|e| eyre!("WebSocket handshake with {peer} failed: {e}"))?;
	// After the handshake, which a timeout would interrupt
	ws.get_mut().set_read_timeout(Some(POLL))?;
	Ok(ws)
}

/// The next binary message, or `None` if nothing arrived within [`POLL`] or a signal (Ctrl-C)
/// interrupted the wait.
fn read(ws: &mut WebSocket<TcpStream>) -> Result<Option<Vec<u8>>> {
	match ws.read() {
		Ok(Message::Binary(data)) => Ok(Some(data.to_vec())),
		Ok(Message::Close(_)) => bail!("The connection was closed"),
		Ok(_) => Ok(None),
		Err(tungstenite::Error::Io(e))
			if matches!(
				e.kind(),
				std::io::ErrorKind::WouldBlock
					| std::io::ErrorKind::TimedOut
					| std::io::ErrorKind::Interrupted
			) =>
		{
			Ok(None)
		}
		Err(e) => Err(e.into()),
	}
}

/// Print any logs in a message from the phone.
fn handle(data: &[u8], logs: bool) {
	if !logs {
		return;
	}
	match Endpoint::split(data) {
		Some((Endpoint::RelayFromWatch, mut packets)) => {
			// [u16 length][u16 endpoint][payload], big-endian, possibly several back to back
			while let Some((header, rest)) = packets.split_first_chunk::<4>() {
				let len = usize::from(u16::from_be_bytes([header[0], header[1]]));
				let endpoint = u16::from_be_bytes([header[2], header[3]]);
				let Some((payload, rest)) = rest.split_at_checked(len) else {
					break;
				};
				if endpoint == APP_LOGS {
					print_app_log(payload);
				}
				packets = rest;
			}
		}
		Some((Endpoint::PhoneAppLog, message)) => {
			println!("[phone] {}", String::from_utf8_lossy(message));
		}
		_ => {}
	}
}

/// An app log line: uuid (16), timestamp u32, level u8, message length u8, line u16, filename (16,
/// NUL-padded), message.
fn print_app_log(payload: &[u8]) {
	let Some(fields) = payload.get(..40) else {
		return;
	};
	let level = match fields[20] {
		0..=1 => "ERROR",
		2..=50 => "WARN",
		51..=100 => "INFO",
		101..=200 => "DEBUG",
		_ => "VERBOSE",
	};
	let message_len = usize::from(fields[21]);
	let line = u16::from_be_bytes([fields[22], fields[23]]);
	let filename = &fields[24..40];
	let filename = &filename[..filename.iter().position(|&b| b == 0).unwrap_or(16)];
	let message = &payload[40..payload.len().min(40 + message_len)];
	println!(
		"{level:>7} {}:{line}> {}",
		String::from_utf8_lossy(filename),
		String::from_utf8_lossy(message)
	);
}

fn set_log_shipping(ws: &mut WebSocket<TcpStream>, on: bool) -> Result<()> {
	let packet = pebble_packet(APP_LOGS, &[u8::from(on)]);
	ws.send(Message::binary(frame(Endpoint::RelayToWatch, &packet)))?;
	Ok(())
}

fn frame(endpoint: Endpoint, payload: &[u8]) -> Vec<u8> {
	let mut frame = Vec::with_capacity(1 + payload.len());
	frame.push(endpoint as u8);
	frame.extend_from_slice(payload);
	frame
}

fn pebble_packet(endpoint: u16, payload: &[u8]) -> Vec<u8> {
	let len = u16::try_from(payload.len()).expect("Pebble Protocol payloads fit a u16");
	let mut packet = Vec::with_capacity(4 + payload.len());
	packet.extend_from_slice(&len.to_be_bytes());
	packet.extend_from_slice(&endpoint.to_be_bytes());
	packet.extend_from_slice(payload);
	packet
}
