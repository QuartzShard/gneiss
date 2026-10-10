use std::{collections::HashMap, path::Path};

use color_eyre::{Result, eyre::bail};
use gneiss_types::Platform;
use quote::quote;

use crate::exported_symbols::{Export, ExportedSymbols, Flag, Function};

fn walk_exports<'ex>(
	exports: &'ex mut [Export],
	deprecated: bool,
) -> Box<dyn Iterator<Item = &'ex mut Function> + 'ex> {
	Box::new(exports.iter_mut().flat_map(move |export| match export {
		Export::Function(function) => {
			if !function.deprecated.is_set() && deprecated {
				function.deprecated = Flag::Set(deprecated);
			}
			Box::new(std::iter::once(function)) as Box<dyn Iterator<Item = &mut Function>>
		}
		Export::Group(group) => {
			walk_exports(&mut group.exports, deprecated || group.deprecated.is_set())
		}
		_ => Box::new(std::iter::empty()) as Box<dyn Iterator<Item = &mut Function>>,
	}))
}

pub fn map_bindgen_to_shimmed(
	bindings: &bindgen::Bindings,
	vendor: &Path,
	platform: Platform,
) -> Result<syn::File> {
	let mut buf = vec![];
	bindings.write(Box::new(&mut buf))?;
	let mut bindings = syn::parse_file(&String::from_utf8_lossy(&buf))?;
	let mut symbols: ExportedSymbols = serde_json::from_reader(std::io::BufReader::new(
		std::fs::File::open(vendor.join("exported_symbols.json"))?,
	))?;

	let mut syscall_vtable = build_syscall_vtable_map(&mut symbols)?;
	let mapped_items: Vec<syn::Item> = itertools::process_results(
		bindings
			.items
			.into_iter()
			.map(|i| map_item(i, &mut syscall_vtable, platform)),
		|fns| fns.flatten().collect(),
	)?;

	if !syscall_vtable.is_empty() {
		log::warn!(
			"exported_fn list not fully drained! {} entries remain",
			syscall_vtable.len()
		);
		for (_, fun) in syscall_vtable.values() {
			log::warn!("  -> {}", fun.name);
		}
	}

	// Replace the AST with our modified one, unparse it, and write out to sys crate
	bindings.items = mapped_items;
	Ok(bindings)
}

fn build_syscall_vtable_map(
	symbols: &mut ExportedSymbols,
) -> Result<HashMap<&str, (usize, &Function)>> {
	let mut exported_fns = walk_exports(&mut symbols.exports, false)
		// Drop anything added "in the future" (999 is used as a NYI sentinel and stubs are marked
		// by added > current)
		.filter(|f| if f.internal { 999 } else { f.added_revision } <= symbols.revision)
		// Reborrow as immutible.
		.map(|f| &*f)
		.collect::<Vec<_>>();

	// Sort by Revision, then alphabetically (so new revision additions are append-only
	exported_fns.sort_by(|a, b| {
		a.added_revision.cmp(&b.added_revision).then_with(|| {
			a.sort_name
				.as_deref()
				.unwrap_or(&a.name)
				.cmp(b.sort_name.as_deref().unwrap_or(&b.name))
		})
	});

	// Now we're in order, assign indexes, then remove anything we don't need a shim for but had to
	// occupy a slot, like removed/deprecated fns.
	let slot_count = exported_fns.len();
	let to_skip = |func: &Function| -> bool {
		// The allocator family and mem* are `skipDefinition` in the SDK but we still want shims
		// for them: gneiss's GlobalAlloc binds the former, gneiss-sys's mem module the latter.
		const FORCE_EMIT: &[&str] = &["malloc", "free", "realloc", "memcpy", "memmove", "memset"];
		func.removed.is_set()
			|| func.deprecated.is_set()
			|| (func.skip_definition && !FORCE_EMIT.contains(&func.name.as_str()))
	};
	let removed_count = exported_fns.iter().filter(|f| to_skip(f)).count();
	let exported_fns: HashMap<&str, (usize, &Function)> = exported_fns
		.iter()
		.enumerate()
		.filter_map(|(index, func)| {
			if to_skip(func) {
				None
			} else {
				Some((&*func.name, (index, *func)))
			}
		})
		.collect();

	if exported_fns.len() + removed_count != slot_count {
		bail!(
			"Exported function len mismatch! {slot_count} Slots - {removed_count} Removed != {}",
			exported_fns.len()
		);
	}
	Ok(exported_fns)
}

fn map_item<'f>(
	item: syn::Item,
	exported_fns: &mut HashMap<&'f str, (usize, &'f Function)>,
	platform: Platform,
) -> Result<Vec<syn::Item>> {
	Ok(if let syn::Item::ForeignMod(item_foreign_mod) = item {
		map_mod(item_foreign_mod, exported_fns, platform)?
	} else if let syn::Item::Mod(mut item_mod) = item {
		if let Some((b, items)) = item_mod.content {
			let mapped_items: Vec<syn::Item> = itertools::process_results(
				items
					.into_iter()
					.map(|i| map_item(i, exported_fns, platform)),
				|fns| fns.flatten().collect(),
			)?;
			item_mod.content = Some((b, mapped_items));
			vec![syn::Item::Mod(item_mod)]
		} else {
			Vec::new()
		}
	} else {
		vec![item]
	})
}

fn map_mod(
	mut item_foreign_mod: syn::ItemForeignMod,
	exported_fns: &mut HashMap<&str, (usize, &Function)>,
	platform: Platform,
) -> Result<Vec<syn::Item>> {
	// Pull out any functions we need to shim as ItemFn, pass through the rest of the ItemForeignMod
	// as-is otherpise
	let mut maybe_shimmed = Vec::new();
	let mut passthrough = Vec::new();
	for item in item_foreign_mod.items {
		if let syn::ForeignItem::Fn(ref pebble_fn) = item {
			let shimmed = map_fn(pebble_fn, exported_fns, platform)?;
			maybe_shimmed.push(syn::Item::Fn(shimmed));
		} else {
			passthrough.push(item);
		}
	}
	item_foreign_mod.items = passthrough;
	if !item_foreign_mod.items.is_empty() {
		maybe_shimmed.push(syn::Item::ForeignMod(item_foreign_mod));
	}
	Ok(maybe_shimmed)
}

fn map_fn(
	pebble_fn: &syn::ForeignItemFn,
	exported_fns: &mut HashMap<&str, (usize, &Function)>,
	platform: Platform,
) -> Result<syn::ItemFn> {
	let sig = &pebble_fn.sig;
	let ident: &str = &sig.ident.to_string();
	if let Some((index, exported_fn)) = exported_fns.get(ident) {
		log::info!(target: &format!("xtask::map_shim::{platform}"),"Shimming `{ident}` at slot {index}");
		let fn_ident = &sig.ident;
		let fn_args = &sig.inputs;
		let fn_variadic = &sig.variadic.as_ref().map(|_| quote!(args: ...));
		let fn_ret = &sig.output;

		let fn_doc = format!(
			" {}\ngneiss: Vtable Slot {index}, Offset {}",
			exported_fn,
			index * core::mem::size_of::<u32>()
		);
		let fn_doc = syn::parse_quote!(
			#[doc = #fn_doc]
		);
		let doc_attrs: Vec<syn::Attribute> = pebble_fn
			.attrs
			.iter()
			.filter(|a| a.path().is_ident("doc"))
			.cloned()
			.chain(std::iter::once(fn_doc))
			.collect();
		let link_section = format!(".text.{ident}");
		let index_op = format!("ldr   r1, ={}", index * core::mem::size_of::<u32>());

		let shimmed: syn::ItemFn = syn::parse_quote! {
			#(#doc_attrs)*
			#[unsafe(naked)]
			#[unsafe(link_section = #link_section)]
			pub unsafe extern "C" fn #fn_ident(#fn_args #fn_variadic) #fn_ret {
				core::arch::naked_asm!(
					"push  {{r0-r3}}",
					#index_op,
					"b     {resolver}",
					resolver = sym crate::shim::jump_to_pbl_function,
				)
			}
		};
		pretty_fn_please(&shimmed);
		exported_fns.remove(ident); // Consume the map
		Ok(shimmed)
	} else {
		bail!("Bindgen declared a fn we didn't identify! {ident}")
	}
}

fn pretty_fn_please(fun: &syn::ItemFn) {
	log::debug!(
		"{}",
		prettyplease::unparse(&syn::File {
			shebang:     None,
			frontmatter: None,
			attrs:       vec![],
			items:       vec![syn::Item::Fn(fun.clone())],
		})
	);
}
