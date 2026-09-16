use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

#[derive(darling::FromMeta, Debug, Default)]
struct EntryArgs {
	panic: Option<syn::Path>,
}

pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut errs = darling::Error::accumulator();
	let args = errs
		.handle(
			darling::ast::NestedMeta::parse_meta_list(attr)
				.map_err(darling::Error::from)
				.and_then(|m| EntryArgs::from_list(&m)),
		)
		.unwrap_or_default();

	let mut out = item.clone();
	let input: Option<syn::ItemFn> = errs.handle(syn::parse2(item).map_err(darling::Error::from));

	let handler = args.panic.unwrap_or_else(|| {
		parse_quote! {
			::gneiss::sys::panic::panic_minimal
		}
	});

	out.extend(quote! {
		#[doc = " Holds generated constants naming each resource index"]
		mod resources {
			include!(concat!(env!("OUT_DIR"), "/resources.rs"));
		}
		#[panic_handler]
		fn __gneiss_panic_handler(info: &::core::panic::PanicInfo) -> ! { #handler(info) }

	});

	if let Some(input) = input {
		validate_entry(&input, &mut errs);
		let fn_name = input.sig.ident;

		out.extend(quote! {
			#[unsafe(export_name = "main")]
			#[doc = " Exported entry point, calls registration routines for `#[service]`s and other init chores"]
			pub extern "C" fn __gneiss_entry() {
				#fn_name();
				unsafe { ::gneiss::sys::app_event_loop() };
			}
		});
	}

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}

/// Push a spanned error for every way `func` fails to qualify as an entry fn:
/// must be safe, non-async, non-variadic, non-generic, take no arguments,
/// return `()`, and carry no symbol-export attributes.
fn validate_entry(func: &syn::ItemFn, errs: &mut darling::error::Accumulator) {
	let sig = &func.sig;
	if let syn::Safety::Unsafe(unsafety) = &sig.safety {
		errs.push(
			darling::Error::custom("entry fn must be safe (drop the `unsafe`)").with_span(unsafety),
		);
	}
	if let Some(asyncness) = &sig.asyncness {
		errs.push(
			darling::Error::custom("entry fn cannot be async — there is no executor")
				.with_span(asyncness),
		);
	}
	if let Some(variadic) = &sig.variadic {
		errs.push(darling::Error::custom("entry fn cannot be variadic").with_span(variadic));
	}
	if !sig.generics.params.is_empty() {
		errs.push(
			darling::Error::custom("entry fn cannot have generic parameters")
				.with_span(&sig.generics),
		);
	}
	if !sig.inputs.is_empty() {
		errs.push(darling::Error::custom("entry fn takes no arguments").with_span(&sig.inputs));
	}
	if let syn::ReturnType::Type(_, ty) = &sig.output {
		let is_unit = matches!(&**ty, syn::Type::Tuple(t) if t.elems.is_empty());
		if !is_unit {
			errs.push(
				darling::Error::custom(
					"entry fn must return `()` — the runtime continues after it",
				)
				.with_span(ty),
			);
		}
	}
	for attr in &func.attrs {
		// `#[unsafe(no_mangle)]` parses as a list with path `unsafe`; unwrap it
		let path = if attr.path().is_ident("unsafe") {
			match attr.parse_args::<syn::Meta>() {
				Ok(inner) => inner.path().clone(),
				Err(_) => continue,
			}
		} else {
			attr.path().clone()
		};
		if path.is_ident("no_mangle") || path.is_ident("export_name") {
			errs.push(
				darling::Error::custom(
					"#[entry] exports the entry symbol itself — remove this attribute",
				)
				.with_span(attr),
			);
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn expands_trivial_main() {
		let out = entry(
			quote!(),
			quote!(
				fn main() {}
			),
		);
		println!(
			"{}",
			prettyplease::unparse(
				&syn::parse2(out.clone()).unwrap_or_else(|e| panic!("{e}\n{out}"))
			)
		);
	}
}
