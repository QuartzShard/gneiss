//! Types for writing out manifest.json and appinfo.json
extern crate std;
use std::collections::BTreeMap;

use serde::Serializer;
use unix_time::Instant;
use uuid::Uuid;

use crate::{
	Platform,
	header::{CURRENT_SDK_VERSION, Version},
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct PebbleAppManifest<'m> {
	manifest_version: u8,
	#[serde(serialize_with = "timestamp")]
	generated_at:     Instant,
	generated_by:     &'m str,
	// Just output {}
	debug:            Debug,
	#[serde(rename = "type")]
	manifest_type:    PebbleManifestType,

	application: Option<PebbleBinary<'m>>,
	resources:   Option<PebbleResources<'m>>,
	#[serde(skip_serializing_if = "Option::is_none")]
	worker:      Option<PebbleBinary<'m>>,
	#[serde(rename = "js_tooling", skip_serializing_if = "Option::is_none")]
	js_tooling:  Option<JsTooling>,
	#[serde(rename = "app_layouts", skip_serializing_if = "Option::is_none")]
	app_layouts: Option<&'m str>,
}

#[derive(serde::Serialize, Debug)]
pub struct Debug {}

#[derive(serde::Serialize, Debug)]
pub struct PebbleBinary<'m> {
	#[serde(serialize_with = "timestamp")]
	timestamp:   Instant,
	sdk_version: Version,
	name:        &'m str,
	size:        u64,
	crc:         u32,
}
#[derive(serde::Serialize, Debug)]
pub struct PebbleResources<'m> {
	name:      &'m str,
	#[serde(serialize_with = "timestamp")]
	timestamp: Instant,
	size:      u64,
	crc:       u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "lowercase")]
#[derive(Debug)]
pub enum PebbleManifestType {
	Application,
	Worker,
	Firmware,
}

#[derive(serde::Serialize, Debug)]
pub struct JsTooling {
	bytecode_version: usize,
}

#[cfg(feature = "serde")]
fn timestamp<S: Serializer>(instant: &Instant, ser: S) -> Result<S::Ok, S::Error> {
	ser.serialize_u64(instant.secs())
}

impl PebbleAppManifest<'static> {
	pub fn new(
		size: u64,
		crc: u32,
		resource_size: u64,
		resource_crc: u32,
		timestamp: Instant,
	) -> PebbleAppManifest<'static> {
		PebbleAppManifest {
			manifest_version: 2,
			generated_at:     timestamp,
			generated_by:     "",
			debug:            Debug {},
			manifest_type:    PebbleManifestType::Application,
			application:      Some(PebbleBinary {
				timestamp,
				sdk_version: CURRENT_SDK_VERSION,
				name: "pebble-app.bin",
				size,
				crc,
			}),
			resources:        Some(PebbleResources {
				name: "app_resources.pbpack",
				timestamp,
				size: resource_size,
				crc: resource_crc,
			}),
			worker:           None,
			js_tooling:       None,
			app_layouts:      None,
		}
	}
}

/// Root `appinfo.json` in the `.pbw`.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct PebbleAppInfo<'m> {
	pub uuid:             Uuid,
	pub name:             &'m str,
	pub short_name:       &'m str,
	pub long_name:        &'m str,
	pub display_name:     &'m str,
	pub company_name:     &'m str,
	pub version_label:    &'m str,
	pub sdk_version:      &'m str,
	pub project_type:     &'m str,
	#[serde(rename = "enableMultiJS")]
	pub enable_multi_js:  bool,
	pub target_platforms: &'m [Platform],
	pub watchapp:         WatchApp,
	pub capabilities:     &'m [&'m str],
	pub resources:        AppResources<'m>,
	pub app_keys:         BTreeMap<&'m str, u32>,
	pub message_keys:     BTreeMap<&'m str, u32>,
}

#[derive(serde::Serialize, Debug)]
pub struct AppResources<'m> {
	pub media: &'m [MediaResource<'m>],
}

/// One `resources.media` entry. Minimal for now; grows per resource type (fonts
/// gain `characterRegex`/`trackingAdjust`, bitmaps `menuIcon`/`storageFormat`, …)
/// once the pbpack pipeline exists.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct MediaResource<'m> {
	#[serde(rename = "type")]
	pub kind: &'m str,
	pub name: &'m str,
	pub file: &'m str,
}

#[derive(serde::Serialize, Debug)]
pub struct WatchApp {
	pub watchface: bool,
}

impl<'m> PebbleAppInfo<'m> {
	/// Build an appinfo from the caller's identity data, defaulting the
	/// boilerplate (native, non-JS, no capabilities, no resources). The four
	/// name fields all take `name`; override by struct literal if you need
	/// distinct short/long/display names.
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		uuid: Uuid,
		name: &'m str,
		company_name: &'m str,
		version_label: &'m str,
		target_platforms: &'m [Platform],
		watchface: bool,
		app_keys: BTreeMap<&'m str, u32>,
		message_keys: BTreeMap<&'m str, u32>,
	) -> PebbleAppInfo<'m> {
		PebbleAppInfo {
			uuid,
			name,
			short_name: name,
			long_name: name,
			display_name: name,
			company_name,
			version_label,
			sdk_version: "3",
			project_type: "native",
			enable_multi_js: false,
			target_platforms,
			watchapp: WatchApp { watchface },
			capabilities: &[],
			resources: AppResources { media: &[] },
			app_keys,
			message_keys,
		}
	}
}
