//! `cargo gneiss install`: send an app bundle to a watch, through the phone app's developer
//! connection, or to an emulator (see [`crate::emulator::install`]).
//!
//! The developer connection is a WebSocket protocol (libpebble2's websocket transport):
//! each binary message is an endpoint byte, then its payload. The phone installs a bundle itself;
//! Pebble Protocol packets to and from the watch ride inside relay messages.
use std::{
	io::Read,
	net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream, ToSocketAddrs},
	path::{Path, PathBuf},
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
use uuid::Uuid;

use crate::build_project::platform_elf;

mod adb;
mod blame;

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

/// Pebble Protocol endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PebbleEndpoint {
	Time            = 0x0B,
	WatchVersion    = 0x10,
	PhoneAppVersion = 0x11,
	AppMessage      = 0x30,
	AppRunState     = 0x34,
	/// The shipping switch towards the watch, log lines back
	AppLogs         = 2006,
	AppFetch        = 0x1771,
	BlobDb          = 0xB1DB,
	PutBytes        = 0xBEEF,
}

impl PebbleEndpoint {
	const ALL: [Self; 9] = [
		Self::Time,
		Self::WatchVersion,
		Self::PhoneAppVersion,
		Self::AppMessage,
		Self::AppRunState,
		Self::AppLogs,
		Self::AppFetch,
		Self::BlobDb,
		Self::PutBytes,
	];

	pub fn from_raw(raw: u16) -> Option<Self> {
		Self::ALL
			.into_iter()
			.find(|endpoint| *endpoint as u16 == raw)
	}
}

const WATCH_VERSION_REQUEST: u8 = 0;
const WATCH_VERSION_RESPONSE: u8 = 1;

/// The start of a watch version response: the command, then the running firmware's metadata
/// (PebbleOS's `FirmwareMetadata`) as far as its hardware platform.
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
struct VersionResponse {
	command:        u8,
	_timestamp:     [u8; 4],
	_version_tag:   [u8; 32],
	_version_short: [u8; 8],
	_flags:         u8,
	/// PebbleOS's `FirmwareMetadataPlatform`
	hw_platform:    u8,
}
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
	/// Stay connected and print the app's logs until interrupted
	#[arg(long)]
	logs:   bool,
	/// With `--logs`, print every app's logs, not only this one's
	#[arg(long, requires = "logs")]
	all:    bool,
	/// Download the emulator without asking, if it isn't cached
	#[arg(short, long)]
	yes:    bool,
}

#[derive(Debug, Clone, clap::Args)]
#[group(required = true, multiple = false)]
struct Target {
	/// The phone's address, `host` or `host:port`, with the Pebble app's developer connection on.
	/// An IPv6 address goes bare or bracketed, `[addr]:port` to give a port. A link-local one
	/// names its interface, as `fe80::1%wlan0`
	#[arg(long)]
	phone:    Option<String>,
	/// The platform of an emulator to install on, started if it isn't running
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
			(_, Some(_), _) => bail!("The emulator runs on Linux and macOS only"),
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

		let stop = if self.logs { ctrl_c()? } else { Arc::default() };
		let mut printer = self
			.logs
			.then(|| LogPrinter::new(Some((&pbw_path, &pbw)), self.all));
		log::info!("Installing {}...", pbw_path.display());
		#[cfg(unix)]
		if let Some(platform) = self.target.emulator {
			if let Some(printer) = &mut printer {
				printer.set_platform(platform);
			}
			return crate::emulator::install::install(platform, &pbw, printer, self.yes, &stop);
		}

		let mut ws = self.target.connect()?;
		if self.logs {
			start_logs(&mut ws)?;
		}
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
			handle(&data, printer.as_mut());
		}
		log::info!("App install succeeded");

		match &mut printer {
			Some(printer) => stream_logs(&mut ws, printer, &stop),
			None => Ok(()),
		}
	}
}

#[derive(Debug, Clone, clap::Parser)]
/// Print a watch's app logs until interrupted, without installing anything
pub struct Logs {
	/// The app whose logs to print and hard faults to put source lines to. Defaults to this
	/// project's `cargo gneiss build` output, if there is one
	pbw:    Option<PathBuf>,
	#[command(flatten)]
	target: Target,
	/// Print every app's logs, not only this one's
	#[arg(long)]
	all:    bool,
}

impl Logs {
	pub fn run(&self) -> Result<()> {
		let bundle = match &self.pbw {
			Some(path) => Some(path.clone()),
			None => default_pbw().ok(),
		};
		let bundle = bundle
			.map(|path| {
				let pbw = std::fs::read(&path)
					.wrap_err_with(|| format!("Can't read {}", path.display()))?;
				Ok::<_, color_eyre::Report>((path, pbw))
			})
			.transpose()?;
		if bundle.is_none() {
			log::info!(
				"No app bundle, so printing every app's logs without source lines for crashes: \
				 name one, or run in its project"
			);
		}
		let mut printer = LogPrinter::new(
			bundle
				.as_ref()
				.map(|(path, pbw)| (path.as_path(), pbw.as_slice())),
			self.all,
		);
		let stop = ctrl_c()?;
		#[cfg(unix)]
		if let Some(platform) = self.target.emulator {
			printer.set_platform(platform);
			return crate::emulator::install::logs(platform, printer, &stop);
		}
		let mut ws = self.target.connect()?;
		start_logs(&mut ws)?;
		stream_logs(&mut ws, &mut printer, &stop)
	}
}

/// A flag Ctrl-C sets, rather than ending the process.
fn ctrl_c() -> Result<Arc<AtomicBool>> {
	let stop = Arc::new(AtomicBool::new(false));
	let flag = Arc::clone(&stop);
	ctrlc::set_handler(move || flag.store(true, Ordering::Relaxed))?;
	Ok(stop)
}

/// Turn on shipping app logs, and ask the watch's version, for the printer to learn its platform.
fn start_logs(ws: &mut WebSocket<TcpStream>) -> Result<()> {
	set_log_shipping(ws, true)?;
	let request = pebble_packet(PebbleEndpoint::WatchVersion, &[WATCH_VERSION_REQUEST]);
	ws.send(Message::binary(frame(Endpoint::RelayToWatch, &request)))?;
	Ok(())
}

/// Print app logs until Ctrl-C, then turn shipping them off.
fn stream_logs(
	ws: &mut WebSocket<TcpStream>,
	printer: &mut LogPrinter,
	stop: &AtomicBool,
) -> Result<()> {
	log::info!("Printing app logs, Ctrl-C to stop");
	while !stop.load(Ordering::Relaxed) {
		match read(ws) {
			Ok(Some(data)) => handle(&data, Some(&mut *printer)),
			Ok(None) => {}
			Err(e) => {
				log::warn!("{e}");
				return Ok(());
			}
		}
	}
	set_log_shipping(ws, false)
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

/// Print any logs in a message from the phone, and learn the watch's platform from its version.
fn handle(data: &[u8], printer: Option<&mut LogPrinter>) {
	let Some(printer) = printer else {
		return;
	};
	match Endpoint::split(data) {
		Some((Endpoint::RelayFromWatch, mut packets)) => {
			// [u16 length][u16 endpoint][payload], big-endian, possibly several back to back
			while let Some((header, rest)) = packets.split_first_chunk::<4>() {
				let len = usize::from(u16::from_be_bytes([header[0], header[1]]));
				let endpoint = u16::from_be_bytes([header[2], header[3]]);
				let Some((payload, rest)) = rest.split_at_checked(len) else {
					break;
				};
				match PebbleEndpoint::from_raw(endpoint) {
					Some(PebbleEndpoint::AppLogs) => printer.print(payload),
					Some(PebbleEndpoint::WatchVersion) => {
						if let Some(platform) = watch_platform(payload) {
							printer.set_platform(platform);
						}
					}
					_ => {}
				}
				packets = rest;
			}
		}
		Some((Endpoint::PhoneAppLog, message)) => {
			printer.print_phone(&String::from_utf8_lossy(message));
		}
		_ => {}
	}
}

/// The platform of the watch whose version response is `payload`.
fn watch_platform(payload: &[u8]) -> Option<Platform> {
	let version: VersionResponse =
		bytemuck::pod_read_unaligned(payload.get(..size_of::<VersionResponse>())?);
	if version.command != WATCH_VERSION_RESPONSE {
		return None;
	}
	Platform::from_hardware(version.hw_platform)
}

/// Prints an app's logs, and puts source lines to its hard faults.
pub struct LogPrinter {
	app: Option<AppInfo>,
	pbw: Option<PathBuf>,
	/// The ELF to look faults up in, once the watch's platform is known
	elf: Option<PathBuf>,
	/// Print every app's logs
	all: bool,
}

impl LogPrinter {
	/// A printer for the logs of `bundle`'s app, given its path and contents, or of every app's if
	/// `all` or there's no bundle.
	pub fn new(bundle: Option<(&Path, &[u8])>, all: bool) -> Self {
		Self {
			app: bundle.and_then(|(_, pbw)| AppInfo::read(pbw)),
			pbw: bundle.map(|(path, _)| path.to_owned()),
			elf: None,
			all,
		}
	}

	pub fn set_platform(&mut self, platform: Platform) {
		self.elf = self.pbw.as_deref().map(|pbw| platform_elf(pbw, platform));
	}

	/// An app log line: uuid (16), timestamp u32, level u8, message length u8, line u16, filename
	/// (16, NUL-padded), message.
	pub fn print(&self, payload: &[u8]) {
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
		let message = String::from_utf8_lossy(&payload[40..payload.len().min(40 + message_len)]);
		// The firmware logs a fault as whichever app is in front by then
		let fault = blame::Fault::parse(&message)
			.filter(|fault| self.app.as_ref().is_some_and(|app| app.uuid == fault.uuid));
		if fault.is_none() && !self.shows(|app| app.uuid.as_bytes() == &fields[..16]) {
			return;
		}
		println!(
			"{level:>7} {}:{line}> {message}",
			String::from_utf8_lossy(filename)
		);
		if let Some(fault) = fault {
			self.blame(&fault);
		}
	}

	/// A log line from the phone: an app's JavaScript logs it as `{long name}:{line} {message}`.
	fn print_phone(&self, message: &str) {
		let ours = |app: &AppInfo| {
			message
				.strip_prefix(&app.name)
				.is_some_and(|rest| rest.starts_with(':'))
		};
		if self.shows(ours) {
			println!("[phone] {message}");
		}
	}

	/// Whether to print a line, given whether it's `from` this printer's app.
	fn shows(&self, from: impl FnOnce(&AppInfo) -> bool) -> bool {
		self.all || self.app.as_ref().is_none_or(from)
	}

	fn blame(&self, fault: &blame::Fault) {
		let lines = if fault.worker {
			vec![String::from(
				"Worker faults aren't blamed: gneiss builds no workers",
			)]
		} else {
			match &self.elf {
				None => vec![String::from(
					"The watch's platform isn't known yet, so there's no ELF to blame this on",
				)],
				Some(elf) if !elf.exists() => vec![format!(
					"No {} to blame this on: rebuild with `cargo gneiss build`",
					elf.display()
				)],
				Some(elf) => blame::blame(elf, fault),
			}
		};
		for line in lines {
			println!("        {line}");
		}
	}
}

/// What tells an app's logs apart, from its bundle's `appinfo.json`.
struct AppInfo {
	uuid: Uuid,
	/// Its `longName`
	name: String,
}

impl AppInfo {
	fn read(pbw: &[u8]) -> Option<Self> {
		let mut zip = zip::ZipArchive::new(std::io::Cursor::new(pbw)).ok()?;
		let mut info = String::new();
		zip.by_name("appinfo.json")
			.ok()?
			.read_to_string(&mut info)
			.ok()?;
		let info: serde_json::Value = serde_json::from_str(&info).ok()?;
		Some(Self {
			uuid: info.get("uuid")?.as_str()?.parse().ok()?,
			name: info.get("longName")?.as_str()?.to_owned(),
		})
	}
}

fn set_log_shipping(ws: &mut WebSocket<TcpStream>, on: bool) -> Result<()> {
	let packet = pebble_packet(PebbleEndpoint::AppLogs, &[u8::from(on)]);
	ws.send(Message::binary(frame(Endpoint::RelayToWatch, &packet)))?;
	Ok(())
}

fn frame(endpoint: Endpoint, payload: &[u8]) -> Vec<u8> {
	let mut frame = Vec::with_capacity(1 + payload.len());
	frame.push(endpoint as u8);
	frame.extend_from_slice(payload);
	frame
}

fn pebble_packet(endpoint: PebbleEndpoint, payload: &[u8]) -> Vec<u8> {
	let len = u16::try_from(payload.len()).expect("Pebble Protocol payloads fit a u16");
	let mut packet = Vec::with_capacity(4 + payload.len());
	packet.extend_from_slice(&len.to_be_bytes());
	packet.extend_from_slice(&(endpoint as u16).to_be_bytes());
	packet.extend_from_slice(payload);
	packet
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn watch_platform_from_version() {
		// As the emery emulator answered: command, timestamp, then "v4.41.2", hardware at 46
		let mut payload = vec![0; 150];
		payload[0] = WATCH_VERSION_RESPONSE;
		payload[5..12].copy_from_slice(b"v4.41.2");
		payload[46] = 0xF5;
		assert_eq!(watch_platform(&payload), Some(Platform::Emery));
		assert_eq!(watch_platform(&payload[..46]), None);
		payload[0] = WATCH_VERSION_REQUEST;
		assert_eq!(watch_platform(&payload), None);
	}
}
