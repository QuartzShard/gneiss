//! Controls for a running emulator, as QEMU frames written to its `pebble.in`. Any number of
//! processes can, alongside the session reading `pebble.out`, such as an `install --logs`.
//!
//! The frames' payloads are big-endian, as pebble-tool's `libpebble2` sends them.
use std::{fs::File, time::Duration};

use color_eyre::{Result, eyre::bail};
use gneiss_types::Platform;

use super::link::{Protocol, write_frame, writer};

/// A click: pebble-tool's
pub const CLICK: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Button {
	Back,
	Up,
	Select,
	Down,
}

impl Button {
	/// The button's bit in QEMU's button frame
	fn mask(self) -> u8 {
		match self {
			Self::Back => 1,
			Self::Up => 2,
			Self::Select => 4,
			Self::Down => 8,
		}
	}
}

/// Press `button` for `hold`, returning once it's released.
pub fn press(platform: Platform, button: Button, hold: Duration) -> Result<()> {
	let mut pipe = open(platform)?;
	write_frame(&mut pipe, Protocol::Button, &[button.mask()])?;
	std::thread::sleep(hold);
	write_frame(&mut pipe, Protocol::Button, &[0])
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Switch {
	On,
	Off,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Tap {
	#[value(name = "x+")]
	XPlus,
	#[value(name = "x-")]
	XMinus,
	#[value(name = "y+")]
	YPlus,
	#[value(name = "y-")]
	YMinus,
	#[value(name = "z+")]
	ZPlus,
	#[value(name = "z-")]
	ZMinus,
}

impl Switch {
	fn byte(self) -> u8 {
		u8::from(matches!(self, Self::On))
	}
}

impl Tap {
	fn axis_sign(self) -> (u8, i8) {
		match self {
			Self::XPlus => (0, 1),
			Self::XMinus => (0, -1),
			Self::YPlus => (1, 1),
			Self::YMinus => (1, -1),
			Self::ZPlus => (2, 1),
			Self::ZMinus => (2, -1),
		}
	}
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum Calibration {
	Uncalibrated,
	Refining,
	#[default]
	Complete,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum TimeFormat {
	#[value(name = "12h")]
	Twelve,
	#[value(name = "24h")]
	TwentyFour,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum ContentSize {
	Small,
	Medium,
	Large,
	#[value(name = "x-large")]
	ExtraLarge,
}

/// The sensors and settings the emulator takes from its host.
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Control {
	/// Show or hide a timeline quick view, the peek that covers the bottom of the screen
	TimelinePeek {
		platform: Platform,
		state:    Switch,
	},
	/// Set the battery charge
	Battery {
		platform: Platform,
		#[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
		percent:  u8,
		#[arg(long)]
		charging: bool,
	},
	/// Connect or disconnect the phone
	Bluetooth {
		platform: Platform,
		state:    Switch,
	},
	/// Tap the watch, along an axis and in a direction
	Tap { platform: Platform, direction: Tap },
	/// Point the compass
	Compass {
		platform:    Platform,
		#[arg(value_parser = clap::value_parser!(u16).range(0..360))]
		degrees:     u16,
		#[arg(long, value_enum, default_value_t)]
		calibration: Calibration,
	},
	/// Show the time in 12 or 24 hours
	TimeFormat {
		platform: Platform,
		format:   TimeFormat,
	},
	/// Set the system text size
	ContentSize {
		platform: Platform,
		size:     ContentSize,
	},
}

impl Control {
	pub fn run(&self) -> Result<()> {
		let (platform, protocol, payload) = match *self {
			Self::TimelinePeek { platform, state } => {
				(platform, Protocol::TimelinePeek, vec![state.byte()])
			}
			Self::Battery {
				platform,
				percent,
				charging,
			} => (platform, Protocol::Battery, vec![
				percent,
				u8::from(charging),
			]),
			Self::Bluetooth { platform, state } => {
				(platform, Protocol::Bluetooth, vec![state.byte()])
			}
			Self::Tap {
				platform,
				direction,
			} => {
				let (axis, sign) = direction.axis_sign();
				(platform, Protocol::Tap, vec![axis, sign.to_be_bytes()[0]])
			}
			Self::Compass {
				platform,
				degrees,
				calibration,
			} => {
				// A full turn is 0x10000
				let heading = (u32::from(degrees) * 0x1_0000).div_ceil(360);
				let mut payload = heading.to_be_bytes().to_vec();
				payload.push(calibration as u8);
				(platform, Protocol::Compass, payload)
			}
			Self::TimeFormat { platform, format } => {
				let is_24h = u8::from(matches!(format, TimeFormat::TwentyFour));
				(platform, Protocol::TimeFormat, vec![is_24h])
			}
			Self::ContentSize { platform, size } => {
				check_content_size(platform, size)?;
				(platform, Protocol::ContentSize, vec![size as u8])
			}
		};
		write_frame(&mut open(platform)?, protocol, &payload)
	}
}

/// emery has no small content size, and only emery has x-large.
fn check_content_size(platform: Platform, size: ContentSize) -> Result<()> {
	match (platform, size) {
		(Platform::Emery, ContentSize::Small) => bail!("emery has no small content size"),
		(Platform::Emery, _)
		| (_, ContentSize::Small | ContentSize::Medium | ContentSize::Large) => Ok(()),
		(_, ContentSize::ExtraLarge) => bail!("Only emery has the x-large content size"),
	}
}

/// `platform`'s `pebble.in`, if its emulator is running.
fn open(platform: Platform) -> Result<File> {
	super::ensure_running(platform)?;
	writer(&super::socket_dir(platform))
}
