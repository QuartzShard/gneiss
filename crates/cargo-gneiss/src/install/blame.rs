//! Put source lines to an app's hard fault. The firmware logs it as the app's own error line,
//! `App fault! {uuid} PC: 0x… LR: 0x…`, the addresses as offsets into the app: its ELF's addresses,
//! since apps link at 0.
use std::{borrow::Cow, path::Path};

use addr2line::gimli;
use object::{Object, ObjectSection};
use uuid::Uuid;

/// A hard fault the firmware reported.
pub struct Fault {
	pub worker: bool,
	pub uuid:   Uuid,
	/// `None` where the firmware logged `???`
	pub pc:     Option<u64>,
	pub lr:     Option<u64>,
}

impl Fault {
	pub fn parse(message: &str) -> Option<Self> {
		let (process, rest) = message.split_once(" fault! ")?;
		let worker = match process {
			"App" => false,
			"Worker" => true,
			_ => return None,
		};
		let (uuid, rest) = rest.split_once(" PC: ")?;
		let (pc, lr) = rest.split_once(" LR: ")?;
		Some(Self {
			worker,
			uuid: uuid.trim_matches(['{', '}']).parse().ok()?,
			pc: address(pc),
			lr: address(lr),
		})
	}
}

fn address(text: &str) -> Option<u64> {
	u64::from_str_radix(text.trim().trim_start_matches("0x"), 16).ok()
}

/// Where gneiss halts the app after logging a panic, so a fault there has its line already.
const PANIC_HALT: &str = "gneiss::fail::log_and_halt";

/// Lines naming the function and source line of `fault`'s registers, from `elf`. Each address's
/// frames run from the code it's in, out through the functions that code was inlined into. None if
/// the fault is gneiss halting after a panic.
pub fn blame(elf: &Path, fault: &Fault) -> Vec<String> {
	let symbols = std::fs::read(elf)
		.map_err(|e| e.to_string())
		.and_then(|data| {
			let symbols = Symbols::new(&data)?;
			let mut lines = Vec::new();
			if let Some(pc) = fault.pc {
				if symbols.function(pc).as_deref() == Some(PANIC_HALT) {
					return Ok(lines);
				}
				lines.extend(symbols.describe("PC", pc, pc));
			}
			// LR is the return address, with the Thumb bit set: the call is just before it
			if let Some(lr) = fault.lr {
				lines.extend(symbols.describe("LR", lr, (lr & !1).saturating_sub(1)));
			}
			Ok(lines)
		});
	symbols.unwrap_or_else(|e| vec![format!("Can't read {}: {e}", elf.display())])
}

type Reader<'a> = gimli::EndianSlice<'a, gimli::RunTimeEndian>;

/// An ELF's line tables and symbols, read as linked. `addr2line`'s own loader applies whatever
/// relocation sections it finds, and apps keep theirs (`--emit-relocs`) already applied, so its
/// addresses would come out shifted by each section's base.
struct Symbols<'a> {
	context: addr2line::Context<Reader<'a>>,
	map:     object::SymbolMap<object::SymbolMapName<'a>>,
}

impl<'a> Symbols<'a> {
	fn new(data: &'a [u8]) -> Result<Self, String> {
		let file = object::File::parse(data).map_err(|e| e.to_string())?;
		let endian = if file.is_little_endian() {
			gimli::RunTimeEndian::Little
		} else {
			gimli::RunTimeEndian::Big
		};
		let dwarf = gimli::Dwarf::load(|id| -> Result<Reader<'a>, gimli::Error> {
			let section = file
				.section_by_name(id.name())
				.and_then(|section| section.data().ok())
				.unwrap_or_default();
			Ok(gimli::EndianSlice::new(section, endian))
		})
		.map_err(|e| e.to_string())?;
		Ok(Self {
			context: addr2line::Context::from_dwarf(dwarf).map_err(|e| e.to_string())?,
			map:     file.symbol_map(),
		})
	}

	/// The function `address` is in, by the symbol table, without its crates' hashes.
	fn function(&self, address: u64) -> Option<String> {
		let symbol = self.map.before(address)?;
		Some(format!("{:#}", rustc_demangle::demangle(symbol.name())))
	}

	/// `register`'s frames at `probe`.
	fn describe(&self, register: &str, address: u64, probe: u64) -> Vec<String> {
		let mut frames: Vec<(String, Option<String>)> = Vec::new();
		if let Ok(mut iter) = self.context.find_frames(probe).skip_all_loads() {
			while let Ok(Some(frame)) = iter.next() {
				let name = frame
					.function
					.as_ref()
					.and_then(|function| function.demangle().ok())
					.map_or_else(|| String::from("?"), Cow::into_owned);
				let place = frame.location.and_then(|location| {
					Some(format!("{}:{}", short_path(location.file?), location.line?))
				});
				frames.push((name, place));
			}
		}
		// Without line tables, the symbol table's name
		if frames.is_empty() {
			let name = self.map.before(probe).map_or_else(
				|| String::from("?"),
				|symbol| addr2line::demangle_auto(symbol.name().into(), None).into_owned(),
			);
			frames.push((name, None));
		}
		let head = format!("{register} {address:#07x}");
		frames
			.into_iter()
			.enumerate()
			.map(|(i, (name, place))| {
				let lead = if i == 0 {
					head.clone()
				} else {
					format!("{:>w$}", "inlined into", w = head.len())
				};
				match place {
					Some(place) => format!("{lead}  {name}  {place}"),
					None => format!("{lead}  {name}"),
				}
			})
			.collect()
	}
}

/// `path` without the compiler's checkout, `/rustc/<commit>/library/`, before its own sources.
fn short_path(path: &str) -> &str {
	path.strip_prefix("/rustc/")
		.and_then(|rest| rest.split_once("/library/"))
		.map_or(path, |(_, file)| file)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse() {
		let fault =
			Fault::parse("App fault! {7f0ef20a-75bd-4b21-bafe-79ec668606fa} PC: 0x1a2c LR: 0x1a01")
				.unwrap();
		assert!(!fault.worker);
		assert_eq!(
			fault.uuid.to_string(),
			"7f0ef20a-75bd-4b21-bafe-79ec668606fa"
		);
		assert_eq!((fault.pc, fault.lr), (Some(0x1A2C), Some(0x1A01)));

		let unknown =
			Fault::parse("Worker fault! {7f0ef20a-75bd-4b21-bafe-79ec668606fa} PC: ??? LR: ???")
				.unwrap();
		assert!(unknown.worker);
		assert_eq!((unknown.pc, unknown.lr), (None, None));

		assert!(Fault::parse("App started").is_none());
	}

	#[test]
	fn short_paths() {
		assert_eq!(
			short_path(
				"/rustc/a30aa9064df6a504b6a323c6884a2f4cfbfff666/library/core/src/option.rs"
			),
			"core/src/option.rs"
		);
		assert_eq!(short_path("./src/main.rs"), "./src/main.rs");
	}
}
