//! The watch and how the app was started: memory, model, firmware, launch details, and the user's
//! preferences.
#[cfg(not(pbl_sdk_frozen))]
use crate::ui::Button;
use crate::{sys, ui::StackText};

/// Bytes free on the app heap.
pub fn heap_free() -> usize {
	unsafe { sys::heap_bytes_free() }
}

/// Bytes in use on the app heap.
pub fn heap_used() -> usize {
	unsafe { sys::heap_bytes_used() }
}

/// Which watch the app is running on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum WatchModel {
	Unknown,
	PebbleOriginal,
	PebbleSteel,
	PebbleTime,
	PebbleTimeSteel,
	/// Pebble Time Round (band width)
	PebbleTimeRound14,
	PebbleTimeRound20,
	Pebble2Hr,
	Pebble2Se,
	/// The Pebble Time 2 announced in 2016, which never shipped. RIP Robert
	PebbleTime2Original,
	Pebble2Duo,
	PebbleTime2,
	PebbleRound2,
}

pub fn watch_model() -> WatchModel {
	match unsafe { sys::watch_info_get_model() } {
		1 => WatchModel::PebbleOriginal,
		2 => WatchModel::PebbleSteel,
		3 => WatchModel::PebbleTime,
		4 => WatchModel::PebbleTimeSteel,
		5 => WatchModel::PebbleTimeRound14,
		6 => WatchModel::PebbleTimeRound20,
		7 => WatchModel::Pebble2Hr,
		8 => WatchModel::Pebble2Se,
		9 => WatchModel::PebbleTime2Original,
		10 => WatchModel::Pebble2Duo,
		11 => WatchModel::PebbleTime2,
		12 => WatchModel::PebbleRound2,
		_ => WatchModel::Unknown,
	}
}

/// The colour and finish of the watch's case, as sold. Band widths are in millimetres.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CaseColour {
	Unknown,
	Black,
	White,
	Red,
	Orange,
	Grey,
	StainlessSteel,
	MatteBlack,
	Blue,
	Green,
	Pink,
	// Pebble Time
	TimeWhite,
	TimeBlack,
	TimeRed,
	// Pebble Time Steel
	TimeSteelSilver,
	TimeSteelBlack,
	TimeSteelGold,
	// Pebble Time Round (strap width)
	TimeRoundSilver14,
	TimeRoundBlack14,
	TimeRoundSilver20,
	TimeRoundBlack20,
	TimeRoundRoseGold14,
	// Pebble 2
	Pebble2SeBlack,
	Pebble2SeWhite,
	Pebble2HrBlack,
	Pebble2HrLime,
	Pebble2HrFlame,
	Pebble2HrWhite,
	Pebble2HrAqua,
	// The Pebble Time 2 announced in 2016, which never shipped. RIP Robert
	PebbleTime2OriginalBlack,
	PebbleTime2OriginalSilver,
	PebbleTime2OriginalGold,
	// Core Devices' Pebble 2 Duo
	Pebble2DuoBlack,
	Pebble2DuoWhite,
	// Core Devices' Pebble Time 2
	PebbleTime2BlackGrey,
	PebbleTime2BlackRed,
	PebbleTime2SilverBlue,
	PebbleTime2SilverGrey,
	// Core Devices' Pebble Round 2 (strap width)
	PebbleRound2Black20,
	PebbleRound2Silver20,
	PebbleRound2Gold14,
	PebbleRound2Silver14,
}

pub fn case_colour() -> CaseColour {
	match unsafe { sys::watch_info_get_color() } {
		1 => CaseColour::Black,
		2 => CaseColour::White,
		3 => CaseColour::Red,
		4 => CaseColour::Orange,
		5 => CaseColour::Grey,
		6 => CaseColour::StainlessSteel,
		7 => CaseColour::MatteBlack,
		8 => CaseColour::Blue,
		9 => CaseColour::Green,
		10 => CaseColour::Pink,
		11 => CaseColour::TimeWhite,
		12 => CaseColour::TimeBlack,
		13 => CaseColour::TimeRed,
		14 => CaseColour::TimeSteelSilver,
		15 => CaseColour::TimeSteelBlack,
		16 => CaseColour::TimeSteelGold,
		17 => CaseColour::TimeRoundSilver14,
		18 => CaseColour::TimeRoundBlack14,
		19 => CaseColour::TimeRoundSilver20,
		20 => CaseColour::TimeRoundBlack20,
		21 => CaseColour::TimeRoundRoseGold14,
		24 => CaseColour::Pebble2SeBlack,
		26 => CaseColour::Pebble2SeWhite,
		25 => CaseColour::Pebble2HrBlack,
		27 => CaseColour::Pebble2HrLime,
		28 => CaseColour::Pebble2HrFlame,
		29 => CaseColour::Pebble2HrWhite,
		30 => CaseColour::Pebble2HrAqua,
		31 => CaseColour::PebbleTime2OriginalBlack,
		32 => CaseColour::PebbleTime2OriginalSilver,
		33 => CaseColour::PebbleTime2OriginalGold,
		34 => CaseColour::Pebble2DuoBlack,
		35 => CaseColour::Pebble2DuoWhite,
		36 => CaseColour::PebbleTime2BlackGrey,
		37 => CaseColour::PebbleTime2BlackRed,
		38 => CaseColour::PebbleTime2SilverBlue,
		39 => CaseColour::PebbleTime2SilverGrey,
		40 => CaseColour::PebbleRound2Black20,
		41 => CaseColour::PebbleRound2Silver20,
		42 => CaseColour::PebbleRound2Gold14,
		43 => CaseColour::PebbleRound2Silver14,
		_ => CaseColour::Unknown,
	}
}

/// A firmware version, `major.minor.patch`. Orders by version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FirmwareVersion {
	pub major: u8,
	pub minor: u8,
	pub patch: u8,
}

pub fn firmware_version() -> FirmwareVersion {
	let v = unsafe { sys::watch_info_get_firmware_version() };
	FirmwareVersion {
		major: v.major,
		minor: v.minor,
		patch: v.patch,
	}
}

/// Why the app was started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LaunchReason {
	System,
	/// From the launcher.
	User,
	Phone,
	/// The app's [`WakeupService`](crate::service::WakeupService) function is called with it at
	/// launch.
	Wakeup,
	Worker,
	QuickLaunch,
	/// [`launch_args`] gives the pin's argument.
	TimelineAction,
	Smartstrap,
}

pub fn launch_reason() -> LaunchReason {
	match unsafe { sys::launch_reason() } {
		1 => LaunchReason::User,
		2 => LaunchReason::Phone,
		3 => LaunchReason::Wakeup,
		4 => LaunchReason::Worker,
		5 => LaunchReason::QuickLaunch,
		6 => LaunchReason::TimelineAction,
		7 => LaunchReason::Smartstrap,
		_ => LaunchReason::System,
	}
}

/// The argument a timeline pin's "open app" action launched the app with, or 0.
pub fn launch_args() -> u32 {
	unsafe { sys::launch_get_args() }
}

/// The button that launched the app, when the user launched it from the launcher or Quick Launch.
#[cfg(not(pbl_sdk_frozen))]
pub fn launch_button() -> Option<Button> {
	match launch_reason() {
		LaunchReason::User | LaunchReason::QuickLaunch => {
			Button::from_raw(unsafe { sys::launch_button() })
		}
		_ => None,
	}
}

/// How a Quick Launch button launched the app.
#[cfg(not(pbl_sdk_frozen))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum QuickLaunch {
	Hold,
	Tap,
	Combo,
}

/// How Quick Launch launched the app, or `None` if it didn't.
#[cfg(not(pbl_sdk_frozen))]
pub fn quick_launch() -> Option<QuickLaunch> {
	match unsafe { sys::launch_get_quick_launch_action() } {
		1 => Some(QuickLaunch::Hold),
		2 => Some(QuickLaunch::Tap),
		3 => Some(QuickLaunch::Combo),
		_ => None,
	}
}

/// Tell the system the app did what it was opened for, so when it exits the watch goes straight
/// back to the watchface rather than the launcher. Call it with `false` to undo that.
#[cfg(not(pbl_platform_aplite))]
pub fn exit_to_watchface(done: bool) {
	let reason = if done {
		sys::AppExitReason_APP_EXIT_ACTION_PERFORMED_SUCCESSFULLY
	} else {
		sys::AppExitReason_APP_EXIT_NOT_SPECIFIED
	};
	unsafe { sys::exit_reason_set(reason) }
}

/// Whether Quiet Time is on: the user doesn't want to be disturbed, such as by vibration.
#[cfg(not(pbl_platform_aplite))]
pub fn quiet_time_active() -> bool {
	unsafe { sys::quiet_time_is_active() }
}

/// The text size the user chose, for the app to scale its UI to.
#[cfg(not(pbl_platform_aplite))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentSize {
	Small,
	Medium,
	Large,
	ExtraLarge,
}

#[cfg(not(pbl_platform_aplite))]
pub fn content_size() -> ContentSize {
	match unsafe { sys::preferred_content_size() } {
		0 => ContentSize::Small,
		2 => ContentSize::Large,
		3 => ContentSize::ExtraLarge,
		_ => ContentSize::Medium,
	}
}

/// How long, in milliseconds, the user wants a result screen shown before it closes itself. The
/// app has to close it.
#[cfg(not(pbl_platform_aplite))]
pub fn result_display_ms() -> u32 {
	unsafe { sys::preferred_result_display_duration() }
}

/// The watch's language, as an ISO locale such as `en_US` or `fr`. It can change while the app
/// runs.
pub fn locale() -> StackText<LOCALE_LEN> {
	let mut text = StackText::new();
	text.fill_c(|buf| {
		// SAFETY: the firmware strncpy()s the locale into the app's buffer of LOCALE_LEN + 1 bytes and
		// returns it, NUL-terminated only if the locale is shorter
		let locale = unsafe {
			core::slice::from_raw_parts(sys::i18n_get_system_locale().cast::<u8>(), LOCALE_LEN + 1)
		};
		buf.copy_from_slice(locale);
	});
	text
}

// The firmware's ISO_LOCALE_LENGTH, less its NUL
const LOCALE_LEN: usize = 5;
