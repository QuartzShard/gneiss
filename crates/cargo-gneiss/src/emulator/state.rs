//! The running emulators, in `.gneiss-emu` in the runtime directory.
use std::{collections::BTreeMap, path::PathBuf};

use color_eyre::{
	Result,
	eyre::{Context, bail},
};
use gneiss_types::Platform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
	pub pid:     u32,
	pub version: String,
}

/// By platform name
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct State(BTreeMap<String, Entry>);

fn path() -> PathBuf {
	super::runtime_dir().join(".gneiss-emu")
}

impl State {
	/// The emulators still running: entries whose process is gone are dropped, with their sockets and pipes.
	pub fn load() -> Result<Self> {
		let path = path();
		let mut state: Self = match std::fs::read(&path) {
			Ok(bytes) => serde_json::from_slice(&bytes)
				.wrap_err_with(|| format!("Can't read {}", path.display()))?,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
			Err(e) => return Err(e).wrap_err_with(|| format!("Can't read {}", path.display())),
		};
		state.0.retain(|name, entry| {
			let live = alive(entry.pid);
			if !live {
				let _ = std::fs::remove_dir_all(super::runtime_dir().join(name));
			}
			live
		});
		Ok(state)
	}

	pub fn save(&self) -> Result<()> {
		let path = path();
		if let Some(dir) = path.parent() {
			std::fs::create_dir_all(dir)?;
		}
		let partial = path.with_extension("partial");
		std::fs::write(&partial, serde_json::to_vec_pretty(self)?)?;
		std::fs::rename(&partial, &path).wrap_err_with(|| format!("Can't write {}", path.display()))
	}

	pub fn get(&self, platform: Platform) -> Option<&Entry> {
		self.0.get(platform.as_str())
	}

	pub fn insert(&mut self, platform: Platform, entry: Entry) {
		self.0.insert(platform.as_str().to_owned(), entry);
	}

	pub fn remove(&mut self, platform: Platform) -> Option<Entry> {
		self.0.remove(platform.as_str())
	}

	pub fn platforms(&self) -> Vec<Platform> {
		self.0.keys().filter_map(|name| name.parse().ok()).collect()
	}
}

pub fn alive(pid: u32) -> bool {
	let Ok(pid) = libc::pid_t::try_from(pid) else {
		return false;
	};
	// SAFETY: signal 0 only checks the process exists
	let found = unsafe { libc::kill(pid, 0) } == 0;
	found || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

pub fn terminate(pid: u32) -> Result<()> {
	let Ok(pid) = libc::pid_t::try_from(pid) else {
		bail!("Bad pid {pid}");
	};
	// SAFETY: sends a signal, touching no memory
	if unsafe { libc::kill(pid, libc::SIGTERM) } != 0 {
		let e = std::io::Error::last_os_error();
		if e.raw_os_error() != Some(libc::ESRCH) {
			return Err(e).wrap_err_with(|| format!("Can't stop pid {pid}"));
		}
	}
	Ok(())
}
