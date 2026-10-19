//! Pebble Protocol with an emulator, over its UART1 FIFOs in QEMU's framing.
//!
//! A QEMU frame is `0xFEED`, a protocol id, the payload length (at most 2048), the payload, then
//! `0xBEEF`, all big-endian. Pebble Protocol travels as SPP frames, split anywhere, so it's
//! reassembled as a stream: `[u16 length][u16 endpoint][payload]`, big-endian.
use std::{
	fs::{File, OpenOptions},
	io::{ErrorKind, Read, Write},
	os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
	path::Path,
	sync::atomic::{AtomicBool, Ordering},
	time::{Duration, Instant, SystemTime},
};

use color_eyre::{
	Result,
	eyre::{Context, bail},
};

use crate::install::{LogPrinter, PebbleEndpoint};

const HEADER: u16 = 0xFEED;
const FOOTER: u16 = 0xBEEF;
const MAX_FRAME: usize = 2048;
/// What a QEMU frame carries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Protocol {
	/// Pebble Protocol
	Spp          = 1,
	Tap          = 2,
	Bluetooth    = 3,
	Compass      = 4,
	Battery      = 5,
	Button       = 8,
	TimeFormat   = 9,
	TimelinePeek = 10,
	ContentSize  = 11,
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum AppMessageCommand {
	Push = 1,
	Nack = 0x7F,
}

/// PhoneAppVersion messages: the watch's request, and the phone's answer
#[derive(Clone, Copy)]
#[repr(u8)]
enum VersionMessage {
	Request  = 0,
	Response = 1,
}

const TIME_SET_UTC: u8 = 3;

/// How often a blocked read wakes up to check for a deadline or Ctrl-C
const POLL: Duration = Duration::from_millis(250);

pub struct Link<'a> {
	/// `pebble.in`
	writer:  File,
	/// `pebble.out`
	reader:  File,
	/// Locked while this link reads
	_lock:   File,
	/// Bytes read, not yet a whole QEMU frame
	frames:  Vec<u8>,
	/// SPP bytes, not yet a whole Pebble Protocol packet
	spp:     Vec<u8>,
	/// Prints app logs as they arrive, if they're shipped
	printer: Option<LogPrinter>,
	/// Set by Ctrl-C
	stop:    &'a AtomicBool,
}

impl<'a> Link<'a> {
	/// Connect to the emulator whose pipes are in `dir`, and tell it a phone is there. Output
	/// waiting from before is dropped. App logs are shipped if there's a `printer` for them.
	pub fn connect(dir: &Path, printer: Option<LogPrinter>, stop: &'a AtomicBool) -> Result<Self> {
		let lock = File::create(dir.join(super::READER_LOCK))?;
		// SAFETY: a valid file descriptor, for as long as `lock` lives
		if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
			bail!("Another session is connected to the emulator, such as an `install --logs`");
		}
		let pipe = dir.join(super::PEBBLE_PIPE);
		let reader = OpenOptions::new()
			.read(true)
			.custom_flags(libc::O_NONBLOCK)
			.open(pipe.with_extension("out"))?;
		let mut link = Self {
			writer: writer(dir)?,
			reader,
			_lock: lock,
			frames: Vec::new(),
			spp: Vec::new(),
			printer,
			stop,
		};
		while link.read(Duration::ZERO)? {}
		link.frames.clear();
		link.send_qemu(Protocol::Bluetooth, &[1])?;
		link.send(PebbleEndpoint::PhoneAppVersion, &phone_app_version())?;
		link.send(PebbleEndpoint::Time, &set_utc()?)?;
		let shipping = u8::from(link.printer.is_some());
		link.send(PebbleEndpoint::AppLogs, &[shipping])?;
		Ok(link)
	}

	pub fn send(&mut self, endpoint: PebbleEndpoint, payload: &[u8]) -> Result<()> {
		let len = u16::try_from(payload.len()).wrap_err("Pebble Protocol payload over 64 KiB")?;
		let mut packet = Vec::with_capacity(4 + payload.len());
		packet.extend_from_slice(&len.to_be_bytes());
		packet.extend_from_slice(&(endpoint as u16).to_be_bytes());
		packet.extend_from_slice(payload);
		for chunk in packet.chunks(MAX_FRAME) {
			self.send_qemu(Protocol::Spp, chunk)?;
		}
		Ok(())
	}

	fn send_qemu(&mut self, protocol: Protocol, payload: &[u8]) -> Result<()> {
		write_frame(&mut self.writer, protocol, payload)
	}

	/// The next Pebble Protocol packet that isn't handled here, or `None` if nothing came within
	/// a poll interval.
	pub fn recv(&mut self) -> Result<Option<(u16, Vec<u8>)>> {
		loop {
			if let Some((endpoint, payload)) = self.next_packet() {
				if !self.handle(endpoint, &payload)? {
					return Ok(Some((endpoint, payload)));
				}
				continue;
			}
			if !self.next_frame() && !self.read(POLL)? {
				return Ok(None);
			}
		}
	}

	/// The next packet for `endpoint`, waiting up to `timeout`. Others are dropped.
	pub fn expect(
		&mut self,
		endpoint: PebbleEndpoint,
		timeout: Duration,
		what: &str,
	) -> Result<Vec<u8>> {
		let deadline = Instant::now() + timeout;
		loop {
			if let Some((from, payload)) = self.recv()?
				&& from == endpoint as u16
			{
				return Ok(payload);
			}
			if self.stop.load(Ordering::Relaxed) {
				bail!("Interrupted");
			}
			if Instant::now() > deadline {
				bail!(
					"The emulator didn't answer the {what} within {} s",
					timeout.as_secs()
				);
			}
		}
	}

	/// Packets a connected phone answers whatever it's doing. Returns whether `endpoint` was one.
	fn handle(&mut self, endpoint: u16, payload: &[u8]) -> Result<bool> {
		match PebbleEndpoint::from_raw(endpoint) {
			Some(PebbleEndpoint::AppLogs) => {
				if let Some(printer) = &self.printer {
					printer.print(payload);
				}
				Ok(true)
			}
			// A push from the app: NACK it, as there's no phone-side app to take it
			Some(PebbleEndpoint::AppMessage) => {
				if let [command, transaction, ..] = *payload
					&& command == AppMessageCommand::Push as u8
				{
					self.send(PebbleEndpoint::AppMessage, &[
						AppMessageCommand::Nack as u8,
						transaction,
					])?;
				}
				Ok(true)
			}
			// The firmware asks once a session opens; the answer was sent on connecting
			Some(PebbleEndpoint::PhoneAppVersion) => {
				if payload.first() == Some(&(VersionMessage::Request as u8)) {
					self.send(PebbleEndpoint::PhoneAppVersion, &phone_app_version())?;
				}
				Ok(true)
			}
			_ => Ok(false),
		}
	}

	/// Read what's waiting into `frames`, waiting up to `wait` for something. Returns whether
	/// anything came.
	fn read(&mut self, wait: Duration) -> Result<bool> {
		let mut poll = libc::pollfd {
			fd:      self.reader.as_raw_fd(),
			events:  libc::POLLIN,
			revents: 0,
		};
		#[allow(clippy::cast_possible_truncation)]
		// SAFETY: one valid pollfd
		if unsafe { libc::poll(&raw mut poll, 1, wait.as_millis() as libc::c_int) } <= 0 {
			return Ok(false);
		}
		let mut buf = [0; 4096];
		match self.reader.read(&mut buf) {
			Ok(0) => bail!("The emulator closed its pipe"),
			Ok(n) => {
				self.frames.extend_from_slice(&buf[..n]);
				Ok(true)
			}
			Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => {
				Ok(false)
			}
			Err(e) => Err(e.into()),
		}
	}

	/// Move the first whole QEMU frame out of `frames`, keeping SPP payloads. Returns whether one
	/// was there.
	fn next_frame(&mut self) -> bool {
		loop {
			let Some(start) = self
				.frames
				.windows(2)
				.position(|w| w == HEADER.to_be_bytes())
			else {
				let keep = usize::from(self.frames.last() == Some(&0xFE));
				self.frames.drain(..self.frames.len() - keep);
				return false;
			};
			self.frames.drain(..start);
			let Some(header) = self.frames.first_chunk::<6>() else {
				return false;
			};
			let protocol = u16::from_be_bytes([header[2], header[3]]);
			let len = usize::from(u16::from_be_bytes([header[4], header[5]]));
			if len > MAX_FRAME {
				self.frames.drain(..2);
				continue;
			}
			let Some(frame) = self.frames.get(..6 + len + 2) else {
				return false;
			};
			if frame[6 + len..] != FOOTER.to_be_bytes() {
				self.frames.drain(..2);
				continue;
			}
			if protocol == Protocol::Spp as u16 {
				self.spp.extend_from_slice(&frame[6..6 + len]);
			}
			self.frames.drain(..6 + len + 2);
			return true;
		}
	}

	fn next_packet(&mut self) -> Option<(u16, Vec<u8>)> {
		let header = self.spp.first_chunk::<4>()?;
		let len = usize::from(u16::from_be_bytes([header[0], header[1]]));
		let endpoint = u16::from_be_bytes([header[2], header[3]]);
		let payload = self.spp.get(4..4 + len)?.to_vec();
		self.spp.drain(..4 + len);
		Some((endpoint, payload))
	}
}

/// `pebble.in` of the emulator whose pipes are in `dir`, to write frames to.
pub fn writer(dir: &Path) -> Result<File> {
	let path = dir.join(super::PEBBLE_PIPE).with_extension("in");
	// Non-blocking only to open, so a FIFO nothing reads is an error rather than a hang
	let file = OpenOptions::new()
		.write(true)
		.custom_flags(libc::O_NONBLOCK)
		.open(&path)
		.wrap_err_with(|| format!("Can't open {}: is the emulator running?", path.display()))?;
	// SAFETY: a valid file descriptor; clears O_NONBLOCK, so writes wait for room
	unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, 0) };
	Ok(file)
}

/// Write one QEMU frame in one `write`: at most 2056 bytes, under `PIPE_BUF`, so frames from
/// different processes never interleave.
pub fn write_frame(writer: &mut File, protocol: Protocol, payload: &[u8]) -> Result<()> {
	let mut frame = Vec::with_capacity(8 + payload.len());
	frame.extend_from_slice(&HEADER.to_be_bytes());
	frame.extend_from_slice(&(protocol as u16).to_be_bytes());
	frame.extend_from_slice(&u16::try_from(payload.len())?.to_be_bytes());
	frame.extend_from_slice(payload);
	frame.extend_from_slice(&FOOTER.to_be_bytes());
	let written = writer.write(&frame)?;
	if written != frame.len() {
		bail!(
			"Wrote {written} of a {}-byte frame to the emulator",
			frame.len()
		);
	}
	Ok(())
}

/// The answer to the firmware's phone app version request: an Android phone app with every
/// protocol capability, as PebbleOS's own `pbl` tool gives it.
fn phone_app_version() -> Vec<u8> {
	// Neither read by firmware since 3.0
	const PROTOCOL_VERSION: u32 = 0xFFFF_FFFF;
	const SESSION_CAPABILITIES: u32 = 0x8000_0000;
	/// The phone's OS, in the low three bits
	const ANDROID: u32 = 2;
	const RESPONSE_VERSION: u8 = 2;
	/// major, minor, bugfix
	const APP_VERSION: [u8; 3] = [3, 0, 0];
	const ALL_CAPABILITIES: u64 = u64::MAX;

	let mut payload = vec![VersionMessage::Response as u8];
	payload.extend_from_slice(&PROTOCOL_VERSION.to_be_bytes());
	payload.extend_from_slice(&SESSION_CAPABILITIES.to_be_bytes());
	payload.extend_from_slice(&ANDROID.to_be_bytes());
	payload.push(RESPONSE_VERSION);
	payload.extend_from_slice(&APP_VERSION);
	payload.extend_from_slice(&ALL_CAPABILITIES.to_be_bytes());
	payload
}

/// Set the watch's time: UTC, the local offset in minutes, and the zone's name.
fn set_utc() -> Result<Vec<u8>> {
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)?
		.as_secs();
	let (offset_secs, zone) = local_zone(now);
	let mut payload = vec![TIME_SET_UTC];
	payload.extend_from_slice(&u32::try_from(now)?.to_be_bytes());
	payload.extend_from_slice(&i16::try_from(offset_secs / 60)?.to_be_bytes());
	let zone = &zone.as_bytes()[..zone.len().min(u8::MAX.into())];
	payload.push(u8::try_from(zone.len())?);
	payload.extend_from_slice(zone);
	Ok(payload)
}

/// The local UTC offset in seconds at `unix` time, and the zone's abbreviation.
fn local_zone(unix: u64) -> (i64, String) {
	let Ok(time) = libc::time_t::try_from(unix) else {
		return (0, "UTC".to_owned());
	};
	// SAFETY: tm is plain data, and localtime_r only writes to it
	let mut tm: libc::tm = unsafe { std::mem::zeroed() };
	// SAFETY: both pointers are valid for the call
	if unsafe { libc::localtime_r(&raw const time, &raw mut tm) }.is_null() {
		return (0, "UTC".to_owned());
	}
	let zone = if tm.tm_zone.is_null() {
		String::new()
	} else {
		// SAFETY: localtime_r sets tm_zone to a static NUL-terminated string
		unsafe { std::ffi::CStr::from_ptr(tm.tm_zone) }
			.to_string_lossy()
			.into_owned()
	};
	(tm.tm_gmtoff, zone)
}
