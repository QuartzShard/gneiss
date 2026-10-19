//! Install a bundle on an emulator, as the phone app does on a watch.
//!
//! 1. Insert the app's metadata into the watch's app database (BlobDB).
//! 2. Launch it (AppRunState). The watch has no binary yet, so it asks for one (AppFetch).
//! 3. Send the binary, resources and worker (PutBytes), then wait for the watch to say the app is
//!    running.
use std::{
	io::Read,
	sync::atomic::{AtomicBool, Ordering},
	time::Duration,
};

use color_eyre::{
	Result,
	eyre::{Context, OptionExt, bail},
};
use gneiss_types::{Platform, header::PebbleProcessInfo, stm_crc32::crc};

use super::link::Link;
use crate::install::APP_LOGS;

const BLOB_DB: u16 = 0xB1DB;
const APP_RUN_STATE: u16 = 0x34;
const APP_FETCH: u16 = 0x1771;
const PUT_BYTES: u16 = 0xBEEF;
const BLOB_DB_APPS: u8 = 2;
const PUT_BYTES_CHUNK: usize = 2000;
/// As long as the firmware waits between PutBytes messages
const TIMEOUT: Duration = Duration::from_secs(30);
/// After the app reports running, until its launch animation ends and it takes button presses:
/// 300 ms was enough on emery
const LAUNCH_SETTLE: Duration = Duration::from_millis(500);

/// PutBytes object types; `| 0x80` marks a transfer for an app fetch, by install id
#[derive(Clone, Copy)]
enum Object {
	Resources = 4,
	Binary    = 5,
	Worker    = 7,
}

/// The parts of a bundle for one platform.
struct App {
	binary:    Vec<u8>,
	resources: Option<Vec<u8>>,
	worker:    Option<Vec<u8>>,
	header:    PebbleProcessInfo,
}

pub fn install(
	platform: Platform,
	pbw: &[u8],
	logs: bool,
	yes: bool,
	stop: &AtomicBool,
) -> Result<()> {
	let app = App::read(pbw, platform)?;
	let built = app.header.sdk_version;
	let knows = platform.config().sdk_version;
	if (built.major, built.minor) > (knows.major, knows.minor) {
		bail!(
			"The app is built for SDK {}.{}, but the {platform} emulator's firmware ({}) only runs \
			 apps up to {}.{}: update cargo-gneiss",
			built.major,
			built.minor,
			super::assets::version(platform),
			knows.major,
			knows.minor
		);
	}
	let dir = super::start(platform, false, yes)?;
	let mut link = Link::connect(&dir, logs, stop)?;
	if logs {
		link.send(APP_LOGS, &[1])?;
	}

	let uuid = bytemuck::bytes_of(&app.header.uuid).to_vec();
	insert_metadata(&mut link, &app.header)?;
	let mut start = vec![1];
	start.extend_from_slice(&uuid);
	link.send(APP_RUN_STATE, &start)?;

	let fetch = link.expect(APP_FETCH, TIMEOUT, "app launch")?;
	let (Some(1), Some(requested), Some(id)) = (fetch.first(), fetch.get(1..17), fetch.get(17..21))
	else {
		bail!("The emulator sent an unexpected app fetch message");
	};
	if requested != uuid {
		link.send(APP_FETCH, &[1, 3])?;
		bail!("The emulator asked for a different app");
	}
	let install_id = u32::from_le_bytes(id.try_into()?);
	link.send(APP_FETCH, &[1, 1])?;

	put_bytes(&mut link, Object::Binary, &app.binary, install_id)?;
	if let Some(resources) = &app.resources {
		put_bytes(&mut link, Object::Resources, resources, install_id)?;
	}
	if let Some(worker) = &app.worker {
		put_bytes(&mut link, Object::Worker, worker, install_id)?;
	}
	wait_until_running(&mut link, &uuid)?;
	std::thread::sleep(LAUNCH_SETTLE);
	log::info!("App install succeeded");

	if logs {
		log::info!("Printing app logs, Ctrl-C to stop");
		while !stop.load(Ordering::Relaxed) {
			link.recv()?;
		}
		link.send(APP_LOGS, &[0])?;
	}
	Ok(())
}

impl App {
	fn read(pbw: &[u8], platform: Platform) -> Result<Self> {
		let mut zip = zip::ZipArchive::new(std::io::Cursor::new(pbw)).wrap_err("Not a .pbw")?;
		let manifest: serde_json::Value = serde_json::from_slice(
			&read_file(&mut zip, &format!("{platform}/manifest.json"))?
				.ok_or_eyre(format!("The bundle has no {platform} build"))?,
		)?;
		let name = |part: &str| {
			manifest
				.get(part)
				.and_then(|part| part.get("name"))
				.and_then(serde_json::Value::as_str)
				.map(|name| format!("{platform}/{name}"))
		};
		let mut part = |file: Option<String>| -> Result<Option<Vec<u8>>> {
			match file {
				Some(file) => read_file(&mut zip, &file),
				None => Ok(None),
			}
		};
		let binary = part(name("application"))?
			.ok_or_eyre(format!("The bundle's {platform} build has no binary"))?;
		let resources = part(name("resources"))?;
		let worker = part(name("worker"))?;
		let header = binary
			.get(..size_of::<PebbleProcessInfo>())
			.map(bytemuck::pod_read_unaligned)
			.ok_or_eyre("The app binary is shorter than its header")?;
		Ok(Self {
			binary,
			resources,
			worker,
			header,
		})
	}
}

fn read_file(
	zip: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
	name: &str,
) -> Result<Option<Vec<u8>>> {
	let mut file = match zip.by_name(name) {
		Ok(file) => file,
		Err(zip::result::ZipError::FileNotFound) => return Ok(None),
		Err(e) => return Err(e).wrap_err_with(|| format!("Can't read {name} from the bundle")),
	};
	let mut data = Vec::new();
	file.read_to_end(&mut data)?;
	Ok(Some(data))
}

/// Insert the app database entry: little-endian, from the app's header.
fn insert_metadata(link: &mut Link, header: &PebbleProcessInfo) -> Result<()> {
	let uuid = bytemuck::bytes_of(&header.uuid);
	let mut metadata = uuid.to_vec();
	metadata.extend_from_slice(&header.flags.to_le_bytes());
	metadata.extend_from_slice(&header.icon_resource_id.to_le_bytes());
	metadata.extend_from_slice(&[
		header.process_version.major,
		header.process_version.minor,
		header.sdk_version.major,
		header.sdk_version.minor,
		0,
		0,
	]);
	let mut name = [0; 96];
	let header_name = bytemuck::cast_slice::<_, u8>(&header.name);
	name[..header_name.len()].copy_from_slice(header_name);
	metadata.extend_from_slice(&name);

	let token: u16 = rand_token();
	let mut insert = vec![1];
	insert.extend_from_slice(&token.to_le_bytes());
	insert.push(BLOB_DB_APPS);
	insert.push(16);
	insert.extend_from_slice(uuid);
	insert.extend_from_slice(&u16::try_from(metadata.len())?.to_le_bytes());
	insert.extend_from_slice(&metadata);
	link.send(BLOB_DB, &insert)?;

	loop {
		let response = link.expect(BLOB_DB, TIMEOUT, "app database insert")?;
		match response[..] {
			[a, b, status] if u16::from_le_bytes([a, b]) == token => {
				return match status {
					1 => Ok(()),
					status => bail!(
						"The emulator refused the app's database entry (BlobDB status {status})"
					),
				};
			}
			_ => {}
		}
	}
}

/// A token to match a BlobDB reply to its request.
fn rand_token() -> u16 {
	let nanos = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map_or(0, |d| d.subsec_nanos());
	u16::try_from(nanos & 0xFFFF).unwrap_or_default()
}

/// Wait for the watch to say the app is running. Until then, the app fetch screen is in front, and
/// the watch ignores launches, such as the next install's.
fn wait_until_running(link: &mut Link, uuid: &[u8]) -> Result<()> {
	const RUNNING: u8 = 1;
	loop {
		let update = link.expect(APP_RUN_STATE, TIMEOUT, "app launch")?;
		if update.first() == Some(&RUNNING) && update.get(1..17) == Some(uuid) {
			return Ok(());
		}
	}
}

/// Send `object` through PutBytes: init, chunks, commit with its CRC, install. Big-endian.
fn put_bytes(link: &mut Link, kind: Object, object: &[u8], install_id: u32) -> Result<()> {
	let mut init = vec![1];
	init.extend_from_slice(&u32::try_from(object.len())?.to_be_bytes());
	init.push(kind as u8 | 0x80);
	init.extend_from_slice(&install_id.to_be_bytes());
	let cookie = put_bytes_step(link, &init, "PutBytes start")?;

	for chunk in object.chunks(PUT_BYTES_CHUNK) {
		let mut put = vec![2];
		put.extend_from_slice(&cookie.to_be_bytes());
		put.extend_from_slice(&u32::try_from(chunk.len())?.to_be_bytes());
		put.extend_from_slice(chunk);
		put_bytes_step(link, &put, "PutBytes data")?;
	}

	let mut commit = vec![3];
	commit.extend_from_slice(&cookie.to_be_bytes());
	commit.extend_from_slice(&crc(object).to_be_bytes());
	put_bytes_step(link, &commit, "PutBytes commit")?;

	let mut install = vec![5];
	install.extend_from_slice(&cookie.to_be_bytes());
	put_bytes_step(link, &install, "PutBytes install")?;
	Ok(())
}

/// Send one PutBytes message, and return the cookie from the watch's ACK.
fn put_bytes_step(link: &mut Link, message: &[u8], what: &str) -> Result<u32> {
	link.send(PUT_BYTES, message)?;
	let response = link.expect(PUT_BYTES, TIMEOUT, what)?;
	match response[..] {
		[1, a, b, c, d] => Ok(u32::from_be_bytes([a, b, c, d])),
		[2, ..] => bail!("The emulator refused the {what}"),
		_ => bail!("The emulator sent an unexpected answer to the {what}"),
	}
}
