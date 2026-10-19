//! Whether an app's ELF links `core::fmt`, and what pulls it in.
use std::collections::BTreeSet;

use color_eyre::eyre::Result;
use object::{Object, ObjectSection, ObjectSymbol, RelocationTarget, SectionKind, SymbolKind};

/// `core::fmt` linked into an app.
pub struct FmtReport {
	/// The size of its functions
	pub bytes:   u64,
	/// Functions outside `core::fmt` that call into it, demangled without hashes
	pub callers: Vec<String>,
}

fn is_fmt(name: &str) -> bool {
	name.starts_with("core::fmt::")
		|| name.starts_with("alloc::fmt::")
		|| name.contains(" as core::fmt::")
}

/// What of `core::fmt` the ELF at `elf_bytes` links, or `None` if it links none. Finds callers
/// through the relocations `--emit-relocs` keeps.
pub fn fmt_report(elf_bytes: &[u8]) -> Result<Option<FmtReport>> {
	let elf = object::File::parse(elf_bytes)?;
	// Functions by address: (start, end, demangled name, symbol index)
	let mut functions: Vec<(u64, u64, String, usize)> = elf
		.symbols()
		.filter(|sym| sym.kind() == SymbolKind::Text && sym.size() > 0)
		.filter_map(|sym| {
			let name = format!("{:#}", rustc_demangle::demangle(sym.name().ok()?));
			// Thumb addresses carry bit 0
			let start = sym.address() & !1;
			Some((start, start + sym.size(), name, sym.index().0))
		})
		.collect();
	functions.sort_by_key(|f| f.0);

	let bytes: u64 = functions
		.iter()
		.filter(|f| is_fmt(&f.2))
		.map(|f| f.1 - f.0)
		.sum();
	if bytes == 0 {
		return Ok(None);
	}

	let fmt_symbols: BTreeSet<usize> = functions
		.iter()
		.filter(|f| is_fmt(&f.2))
		.map(|f| f.3)
		.collect();
	let mut callers = BTreeSet::new();
	for section in elf.sections().filter(|s| s.kind() == SectionKind::Text) {
		for (offset, reloc) in section.relocations() {
			let RelocationTarget::Symbol(target) = reloc.target() else {
				continue;
			};
			if !fmt_symbols.contains(&target.0) {
				continue;
			}
			// A linked executable's relocations carry absolute addresses, an object's section offsets
			let at = if elf.kind() == object::ObjectKind::Executable {
				offset
			} else {
				section.address() + offset
			};
			let caller = functions
				.iter()
				.rev()
				.find(|f| f.0 <= at && at < f.1)
				.map(|f| &f.2);
			if let Some(caller) = caller.filter(|c| !is_fmt(c)) {
				callers.insert(caller.clone());
			}
		}
	}
	Ok(Some(FmtReport {
		bytes,
		callers: callers.into_iter().collect(),
	}))
}
