//! Screenshots through QEMU's monitor, a text console on its own socket.
use std::{
	io::{Read, Write},
	os::unix::net::UnixStream,
	path::Path,
	time::{Duration, Instant},
};

use color_eyre::{
	Result,
	eyre::{Context, bail},
};
use gneiss_types::Platform;

const PROMPT: &str = "(qemu) ";
const TIMEOUT: Duration = Duration::from_secs(10);

/// Save the screen to `path` as a PNG.
pub fn screenshot(platform: Platform, path: &Path) -> Result<()> {
	let path = std::path::absolute(path)?;
	let Some(file) = path.to_str() else {
		bail!("{} isn't UTF-8, so QEMU can't take it", path.display());
	};
	if file.contains(char::is_whitespace) {
		bail!("{file} has whitespace, which QEMU's monitor can't take");
	}
	let output = Monitor::connect(platform)?.run(&format!("screendump {file} -f png"))?;
	if !path.exists() {
		bail!("QEMU didn't save the screenshot: {}", output.trim());
	}
	Ok(())
}

struct Monitor(UnixStream);

impl Monitor {
	fn connect(platform: Platform) -> Result<Self> {
		super::ensure_running(platform)?;
		let path = super::socket_dir(platform).join(super::MONITOR_SOCKET);
		let stream = UnixStream::connect(&path)
			.wrap_err_with(|| format!("Can't connect to {}", path.display()))?;
		stream.set_read_timeout(Some(Duration::from_millis(250)))?;
		let mut monitor = Self(stream);
		read_to_prompt(&mut monitor.0)?;
		Ok(monitor)
	}

	/// Run `command`, and return what it printed.
	fn run(&mut self, command: &str) -> Result<String> {
		self.0.write_all(format!("{command}\n").as_bytes())?;
		let output = read_to_prompt(&mut self.0)?;
		// The monitor echoes the command line first
		Ok(output
			.split_once('\n')
			.map_or(String::new(), |(_, rest)| rest.to_owned()))
	}
}

fn read_to_prompt(monitor: &mut UnixStream) -> Result<String> {
	let deadline = Instant::now() + TIMEOUT;
	let mut seen = Vec::new();
	let mut buf = [0; 1024];
	loop {
		match monitor.read(&mut buf) {
			Ok(0) => bail!("QEMU's monitor closed"),
			Ok(n) => seen.extend_from_slice(&buf[..n]),
			Err(e)
				if matches!(
					e.kind(),
					std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
				) => {}
			Err(e) => return Err(e.into()),
		}
		let text = String::from_utf8_lossy(&seen);
		if let Some(output) = text.strip_suffix(PROMPT) {
			return Ok(output.to_owned());
		}
		if Instant::now() > deadline {
			bail!(
				"QEMU's monitor didn't answer within {} s",
				TIMEOUT.as_secs()
			);
		}
	}
}
