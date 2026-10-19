//! `cargo gneiss emulator`: run a Pebble emulator, `qemu-pebble`, without pebble-tool.
//!
//! Each platform's emulator runs detached, in its own process group, with its serial ports and
//! monitor on Unix sockets in a directory under [`runtime_dir`]. Its SPI flash, which keeps the
//! apps installed on it, lives under [`data_dir`], one per firmware version.
use std::{
	fs::File,
	io::Read,
	os::unix::{net::UnixStream, process::CommandExt},
	path::{Path, PathBuf},
	process::{Child, Command, Stdio},
	time::{Duration, Instant},
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail},
};
use gneiss_types::Platform;

mod assets;
pub mod install;
mod link;
mod state;

use state::{Entry, State};

/// UART1: Pebble Protocol, in QEMU's framing
pub const PEBBLE_SOCKET: &str = "pebble.sock";
/// UART2: the firmware's debug console
const CONSOLE_SOCKET: &str = "console.sock";
pub const MONITOR_SOCKET: &str = "monitor.sock";
const GDB_SOCKET: &str = "gdb.sock";
const LOG: &str = "qemu.log";
const BOOT_TIMEOUT: Duration = Duration::from_secs(10);
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
/// What the firmware prints on its console once it's up, by version
const BOOTED: [&[u8]; 3] = [b"<SDK Home>", b"<Launcher>", b"Ready for communication"];

#[derive(Debug, Clone, clap::Parser)]
/// Start or stop a Pebble emulator
pub struct Emulator {
	#[command(subcommand)]
	command: EmulatorCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum EmulatorCommand {
	/// Start an emulator, unless one is already running
	Start {
		platform: Platform,
		/// Start from a fresh flash image, losing the apps installed on it
		#[arg(long)]
		wipe:     bool,
		/// Download the emulator without asking, if it isn't cached
		#[arg(short, long)]
		yes:      bool,
	},
	/// Stop an emulator, or all of them
	Stop { platform: Option<Platform> },
}

impl Emulator {
	pub fn run(&self) -> Result<()> {
		match &self.command {
			EmulatorCommand::Start {
				platform,
				wipe,
				yes,
			} => start(*platform, *wipe, *yes).map(|_| ()),
			EmulatorCommand::Stop {
				platform: Some(platform),
			} => stop(*platform),
			EmulatorCommand::Stop { platform: None } => {
				for platform in State::load()?.platforms() {
					stop(platform)?;
				}
				Ok(())
			}
		}
	}
}

/// Where a running emulator's sockets are.
pub fn socket_dir(platform: Platform) -> PathBuf {
	runtime_dir().join(platform.as_str())
}

/// Start `platform`'s emulator if it isn't running, and wait for it to boot. Returns the directory
/// with its sockets.
pub fn start(platform: Platform, wipe: bool, yes: bool) -> Result<PathBuf> {
	let mut state = State::load()?;
	let dir = socket_dir(platform);
	if let Some(entry) = state.get(platform) {
		if wipe {
			bail!("The {platform} emulator is running: stop it before wiping its flash");
		}
		let wanted = assets::version(platform);
		if entry.version != wanted {
			bail!(
				"The {platform} emulator running is firmware {}, not {wanted}: stop it with `cargo \
				 gneiss emulator stop {platform}`",
				entry.version
			);
		}
		log::info!(
			"The {platform} emulator is already running ({})",
			entry.version
		);
		return Ok(dir);
	}

	let assets = assets::locate(platform, yes)?;
	let flash = data_dir()?
		.join(&assets.version)
		.join(platform.as_str())
		.join("qemu_spi_flash.bin");
	if wipe || !flash.exists() {
		fresh_flash(&assets.spi_flash, &flash)?;
	}

	if dir.exists() {
		std::fs::remove_dir_all(&dir).wrap_err_with(|| format!("Can't clear {}", dir.display()))?;
	}
	std::fs::create_dir_all(&dir).wrap_err_with(|| format!("Can't create {}", dir.display()))?;
	let log = File::create(dir.join(LOG))?;

	log::info!("Starting the {platform} emulator ({})", assets.version);
	let mut child = Command::new(&assets.qemu)
		.args(qemu_args(platform, &assets.micro_flash, &flash, &dir)?)
		.stdin(Stdio::null())
		.stdout(log.try_clone()?)
		.stderr(log)
		.process_group(0)
		.spawn()
		.wrap_err_with(|| format!("Can't run {}", assets.qemu.display()))?;
	state.insert(platform, Entry {
		pid:     child.id(),
		version: assets.version,
	});
	state.save()?;

	let started = Instant::now();
	if let Err(e) = wait_for_boot(&mut child, &dir) {
		let _ = child.kill();
		let _ = child.wait();
		state.remove(platform);
		state.save()?;
		return Err(e);
	}
	log::info!(
		"The {platform} emulator booted in {:.1} s",
		started.elapsed().as_secs_f32()
	);
	Ok(dir)
}

pub fn stop(platform: Platform) -> Result<()> {
	let mut state = State::load()?;
	let Some(entry) = state.remove(platform) else {
		log::info!("No {platform} emulator is running");
		return Ok(());
	};
	state::terminate(entry.pid)?;
	let deadline = Instant::now() + STOP_TIMEOUT;
	while state::alive(entry.pid) {
		if Instant::now() > deadline {
			bail!("The {platform} emulator (pid {}) didn't stop", entry.pid);
		}
		std::thread::sleep(Duration::from_millis(50));
	}
	state.save()?;
	let dir = socket_dir(platform);
	if dir.exists() {
		std::fs::remove_dir_all(&dir)?;
	}
	log::info!("Stopped the {platform} emulator");
	Ok(())
}

/// How a platform's machine takes its SPI flash
enum Flash {
	/// The SPI flash, as a drive the machine finds by its id
	Spi,
	/// An `mtd` drive
	Mtd,
}

/// pebble-tool's command line for each platform, as given in its `sdk/emulator.py`, for SDKs
/// after 4.9.148 and QEMU 7 or later.
fn qemu_args(
	platform: Platform,
	micro_flash: &Path,
	flash: &Path,
	dir: &Path,
) -> Result<Vec<String>> {
	let socket = |name: &str| -> Result<String> {
		Ok(format!(
			"unix:{},server=on,wait=off",
			option_path(&dir.join(name))?
		))
	};
	let mut args: Vec<String> = [
		"-rtc",
		"base=localtime",
		"-serial",
		"null",
		"-serial",
		&socket(PEBBLE_SOCKET)?,
		"-serial",
		&socket(CONSOLE_SOCKET)?,
		"-kernel",
		path_str(micro_flash)?,
		"-gdb",
		&socket(GDB_SOCKET)?,
		"-monitor",
		&socket(MONITOR_SOCKET)?,
		"-display",
		"sdl,show-cursor=on",
	]
	.map(str::to_owned)
	.into();
	let (machine, cpu, kind, audio) = match platform {
		Platform::Aplite => ("pebble-bb2", "cortex-m3", Flash::Mtd, false),
		Platform::Basalt => ("pebble-snowy-bb", "cortex-m4", Flash::Spi, false),
		Platform::Chalk => ("pebble-s4-bb", "cortex-m4", Flash::Spi, false),
		Platform::Diorite => ("pebble-silk-bb", "cortex-m4", Flash::Mtd, false),
		Platform::Emery => ("pebble-emery", "cortex-m33", Flash::Mtd, true),
		Platform::Flint => ("pebble-flint", "cortex-m4", Flash::Mtd, true),
		Platform::Gabbro => ("pebble-gabbro", "cortex-m33", Flash::Mtd, false),
	};
	args.extend(["-machine", machine, "-cpu", cpu].map(str::to_owned));
	let flash = option_path(flash)?;
	args.extend(match kind {
		Flash::Spi => [
			"-drive".to_owned(),
			format!("if=none,id=spi-flash,file={flash},format=raw"),
		],
		Flash::Mtd => [
			"-drive".to_owned(),
			format!("if=mtd,format=raw,file={flash}"),
		],
	});
	if audio {
		let driver = if cfg!(target_os = "macos") {
			"coreaudio"
		} else {
			"sdl"
		};
		args.extend(["-audio".to_owned(), format!("driver={driver},id=audio0")]);
	}
	Ok(args)
}

fn path_str(path: &Path) -> Result<&str> {
	path.to_str().ok_or_eyre(format!(
		"{} isn't UTF-8, so QEMU can't take it",
		path.display()
	))
}

/// `path` inside a QEMU option list, where a comma is written `,,`.
fn option_path(path: &Path) -> Result<String> {
	Ok(path_str(path)?.replace(',', ",,"))
}

fn fresh_flash(image: &Path, flash: &Path) -> Result<()> {
	log::info!("Copying a fresh flash image to {}", flash.display());
	let parent = flash.parent().ok_or_eyre("Flash image has no directory")?;
	std::fs::create_dir_all(parent)
		.wrap_err_with(|| format!("Can't create {}", parent.display()))?;
	std::fs::copy(image, flash).wrap_err_with(|| format!("Can't write {}", flash.display()))?;
	Ok(())
}

/// Read the console until the firmware says it's up.
fn wait_for_boot(child: &mut Child, dir: &Path) -> Result<()> {
	let deadline = Instant::now() + BOOT_TIMEOUT;
	let failed = |what: &str| {
		let log = std::fs::read_to_string(dir.join(LOG)).unwrap_or_default();
		let tail: Vec<&str> = log.lines().rev().take(20).collect();
		let tail: Vec<&str> = tail.into_iter().rev().collect();
		color_eyre::eyre::eyre!("The emulator {what}. QEMU's output:\n{}", tail.join("\n"))
	};
	let mut console = loop {
		if let Some(status) = child.try_wait()? {
			return Err(failed(&format!("exited ({status})")));
		}
		if let Ok(console) = UnixStream::connect(dir.join(CONSOLE_SOCKET)) {
			break console;
		}
		if Instant::now() > deadline {
			return Err(failed("never opened its console"));
		}
		std::thread::sleep(Duration::from_millis(50));
	};
	console.set_read_timeout(Some(Duration::from_millis(250)))?;
	let mut seen = Vec::new();
	let mut buf = [0; 512];
	loop {
		match console.read(&mut buf) {
			Ok(0) => return Err(failed("closed its console")),
			Ok(n) => seen.extend_from_slice(&buf[..n]),
			Err(e)
				if matches!(
					e.kind(),
					std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
				) => {}
			Err(e) => return Err(e.into()),
		}
		if BOOTED
			.iter()
			.any(|marker| seen.windows(marker.len()).any(|w| w == *marker))
		{
			return Ok(());
		}
		let keep = seen.len().saturating_sub(64);
		seen.drain(..keep);
		if let Some(status) = child.try_wait()? {
			return Err(failed(&format!("exited ({status})")));
		}
		if Instant::now() > deadline {
			return Err(failed(&format!(
				"didn't boot within {} s",
				BOOT_TIMEOUT.as_secs()
			)));
		}
	}
}

/// Per-session files: sockets, logs and the state file
fn runtime_dir() -> PathBuf {
	std::env::var_os("XDG_RUNTIME_DIR")
		.map_or_else(std::env::temp_dir, PathBuf::from)
		.join("cargo-gneiss")
}

/// Files kept between sessions: flash images
fn data_dir() -> Result<PathBuf> {
	let base = match std::env::var_os("XDG_DATA_HOME") {
		Some(dir) => PathBuf::from(dir),
		None => home()?.join(".local/share"),
	};
	Ok(base.join("cargo-gneiss/emulator"))
}

fn home() -> Result<PathBuf> {
	std::env::var_os("HOME")
		.map(PathBuf::from)
		.ok_or_eyre("HOME isn't set")
}
