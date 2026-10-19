//! Pebble Protocol with an emulator, over its UART1 socket in QEMU's framing.
//!
//! A QEMU frame is `0xFEED`, a protocol id, the payload length (at most 2048), the payload, then
//! `0xBEEF`, all big-endian. Pebble Protocol travels as SPP frames, split anywhere, so it's
//! reassembled as a stream: `[u16 length][u16 endpoint][payload]`, big-endian.
use std::{
	io::{ErrorKind, Read, Write},
	os::unix::net::UnixStream,
	path::Path,
	sync::atomic::{AtomicBool, Ordering},
	time::{Duration, Instant, SystemTime},
};

use color_eyre::{
	Result,
	eyre::{Context, bail},
};

use crate::install::{APP_LOGS, print_app_log};

const HEADER: u16 = 0xFEED;
const FOOTER: u16 = 0xBEEF;
const MAX_FRAME: usize = 2048;
const PROTOCOL_SPP: u16 = 1;
const PROTOCOL_BT_CONNECTION: u16 = 3;

pub const PHONE_APP_VERSION: u16 = 0x11;
pub const TIME: u16 = 0x0B;
pub const APP_MESSAGE: u16 = 0x30;

/// How often a blocked read wakes up to check for a deadline or Ctrl-C
const POLL: Duration = Duration::from_millis(250);

pub struct Link<'a> {
	stream: UnixStream,
	/// Bytes read, not yet a whole QEMU frame
	frames: Vec<u8>,
	/// SPP bytes, not yet a whole Pebble Protocol packet
	spp:    Vec<u8>,
	/// Print app logs as they arrive
	logs:   bool,
	/// Set by Ctrl-C
	stop:   &'a AtomicBool,
}

impl<'a> Link<'a> {
	/// Connect to the emulator whose sockets are in `dir`, and tell it a phone is there.
	pub fn connect(dir: &Path, logs: bool, stop: &'a AtomicBool) -> Result<Self> {
		let path = dir.join(super::PEBBLE_SOCKET);
		let stream = UnixStream::connect(&path).wrap_err_with(|| {
			format!(
				"Can't connect to {}: is something else connected to the emulator?",
				path.display()
			)
		})?;
		stream.set_read_timeout(Some(POLL))?;
		let mut link = Self {
			stream,
			frames: Vec::new(),
			spp: Vec::new(),
			logs,
			stop,
		};
		link.send_qemu(PROTOCOL_BT_CONNECTION, &[1])?;
		link.send(PHONE_APP_VERSION, &phone_app_version())?;
		link.send(TIME, &set_utc()?)?;
		Ok(link)
	}

	pub fn send(&mut self, endpoint: u16, payload: &[u8]) -> Result<()> {
		let len = u16::try_from(payload.len()).wrap_err("Pebble Protocol payload over 64 KiB")?;
		let mut packet = Vec::with_capacity(4 + payload.len());
		packet.extend_from_slice(&len.to_be_bytes());
		packet.extend_from_slice(&endpoint.to_be_bytes());
		packet.extend_from_slice(payload);
		for chunk in packet.chunks(MAX_FRAME) {
			self.send_qemu(PROTOCOL_SPP, chunk)?;
		}
		Ok(())
	}

	fn send_qemu(&mut self, protocol: u16, payload: &[u8]) -> Result<()> {
		let mut frame = Vec::with_capacity(8 + payload.len());
		frame.extend_from_slice(&HEADER.to_be_bytes());
		frame.extend_from_slice(&protocol.to_be_bytes());
		frame.extend_from_slice(&u16::try_from(payload.len())?.to_be_bytes());
		frame.extend_from_slice(payload);
		frame.extend_from_slice(&FOOTER.to_be_bytes());
		self.stream.write_all(&frame)?;
		Ok(())
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
			if !self.next_frame() && !self.read()? {
				return Ok(None);
			}
		}
	}

	/// The next packet for `endpoint`, waiting up to `timeout`. Others are dropped.
	pub fn expect(&mut self, endpoint: u16, timeout: Duration, what: &str) -> Result<Vec<u8>> {
		let deadline = Instant::now() + timeout;
		loop {
			if let Some((from, payload)) = self.recv()?
				&& from == endpoint
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
		match endpoint {
			APP_LOGS => {
				if self.logs {
					print_app_log(payload);
				}
				Ok(true)
			}
			// A push from the app: NACK it, as there's no phone-side app to take it
			APP_MESSAGE => {
				if let [1, transaction, ..] = payload {
					self.send(APP_MESSAGE, &[0x7F, *transaction])?;
				}
				Ok(true)
			}
			// The firmware asks once a session opens; the answer was sent on connecting
			PHONE_APP_VERSION => {
				if payload.first() == Some(&0) {
					self.send(PHONE_APP_VERSION, &phone_app_version())?;
				}
				Ok(true)
			}
			_ => Ok(false),
		}
	}

	/// Read what's waiting into `frames`. Returns whether anything came.
	fn read(&mut self) -> Result<bool> {
		let mut buf = [0; 4096];
		match self.stream.read(&mut buf) {
			Ok(0) => bail!("The emulator closed the connection"),
			Ok(n) => {
				self.frames.extend_from_slice(&buf[..n]);
				Ok(true)
			}
			Err(e)
				if matches!(
					e.kind(),
					ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted
				) =>
			{
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
			if protocol == PROTOCOL_SPP {
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

/// The answer to the firmware's phone app version request: an Android phone app with every
/// protocol capability, as PebbleOS's own `pbl` tool gives it.
fn phone_app_version() -> Vec<u8> {
	let mut payload = vec![1];
	payload.extend_from_slice(&0xFFFF_FFFF_u32.to_be_bytes());
	payload.extend_from_slice(&0x8000_0000_u32.to_be_bytes());
	payload.extend_from_slice(&2_u32.to_be_bytes());
	payload.extend_from_slice(&[2, 3, 0, 0]);
	payload.extend_from_slice(&u64::MAX.to_be_bytes());
	payload
}

/// Set the watch's time: UTC, the local offset in minutes, and the zone's name.
fn set_utc() -> Result<Vec<u8>> {
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)?
		.as_secs();
	let (offset_secs, zone) = local_zone(now);
	let mut payload = vec![3];
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
