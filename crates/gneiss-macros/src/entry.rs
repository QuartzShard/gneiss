use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

#[derive(darling::FromMeta, Debug, Default)]
struct EntryArgs {
	global_alloc: Option<syn::Path>,
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

	let input: Option<syn::ItemFn> =
		errs.handle(syn::parse2(item.clone()).map_err(darling::Error::from));
	let mut out = match &input {
		Some(input) => quote! {
			#[inline(never)]
			#input
		},
		None => item,
	};

	let allocator = args.global_alloc.unwrap_or_else(|| {
		parse_quote! {
			::gneiss::alloc::PebbleAlloc
		}
	});

	out.extend(quote! {
		#[global_allocator]
		pub static ALLOC: #allocator = #allocator;
	});

	out.extend(quote! {
		#[doc = " Generated consts naming each bundled resource by index"]
		#[allow(unused)]
		mod resources {
			include!(concat!(env!("OUT_DIR"), "/resources.rs"));
		}

		#[doc = " Generated consts naming each declared message key"]
		#[allow(unused)]
		mod message_keys {
			include!(concat!(env!("OUT_DIR"), "/message_keys.rs"));
		}
	});

	if let Some(input) = input {
		validate_entry(&input, &mut errs);
		let fn_name = input.sig.ident;

		out.extend(quote! {
			#[unsafe(export_name = "main")]
			pub extern "C" fn __gneiss_entry() {
				__gneiss_setup();
				__gneiss_register();
				::gneiss::log::debug!("gneiss init complete, starting event loop...");
				unsafe { ::gneiss::sys::app_event_loop() };
				::gneiss::stack::report();
			}

			#[doc = " Out of line, so its frame is gone before the event loop starts"]
			#[inline(never)]
			fn __gneiss_setup() {
				::gneiss::log::debug!("Constructing UI...");
				::gneiss::log::verbose!("If handlers fail to register next, check your main() returns.");
				::gneiss::log::verbose!("Don't call `app_event_loop()`, that's handled by gneiss");
				let mut ui = ::gneiss::ui::UiGuard::enter();
				let queue = #fn_name(&mut ui);
				drop(queue);
			}

			#[doc = " Service handlers can fire synchronously on subscribe, so this runs once the first"]
			#[doc = " windows are loaded and the guard is released"]
			#[inline(never)]
			fn __gneiss_register() {
				::gneiss::log::debug!("Registering handlers...");
				for service_init in GNEISS_SERVICES {
					service_init()
				}
				for launch_hook in GNEISS_LAUNCH_HOOKS {
					launch_hook()
				}
			}

			#[::gneiss::linkme::distributed_slice]
			#[linkme(crate = ::gneiss::linkme)]
			pub static GNEISS_SERVICES: [fn()];

			#[::gneiss::linkme::distributed_slice]
			#[linkme(crate = ::gneiss::linkme)]
			pub static GNEISS_LAUNCH_HOOKS: [fn()];
		});
	}

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}

/// Push a spanned error for every way `func` fails to qualify as an entry fn:
/// must be safe, non-async, non-variadic, non-generic, take a single `&mut UiGuard`, return a
/// `UiQueue`, and carry no symbol-export attributes.
fn validate_entry(func: &syn::ItemFn, errs: &mut darling::error::Accumulator) {
	let sig = &func.sig;
	if let syn::Safety::Unsafe(unsafety) = &sig.safety {
		errs.push(
			darling::Error::custom("entry fn must be safe (drop the `unsafe`)").with_span(unsafety),
		);
	}
	if let Some(asyncness) = &sig.asyncness {
		errs.push(
			darling::Error::custom("entry fn cannot be async: there is no executor")
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
	if sig.inputs.len() != 1 {
		errs.push(
			darling::Error::custom("entry fn takes a single `&mut UiGuard`").with_span(&sig.ident),
		);
	}
	let returns_unit = match &sig.output {
		syn::ReturnType::Default => true,
		syn::ReturnType::Type(_, ty) => {
			matches!(&**ty, syn::Type::Tuple(t) if t.elems.is_empty())
		}
	};
	if returns_unit {
		errs.push(
			darling::Error::custom(
				"entry fn returns the `UiQueue` gneiss runs once it's done: `-> UiQueue<'_>`",
			)
			.with_span(&sig.ident),
		);
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
					"#[entry] exports the entry symbol itself, remove this attribute",
				)
				.with_span(attr),
			);
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn expand(item: TokenStream) -> String {
		entry(quote!(), item).to_string()
	}

	#[test]
	fn expands_main() {
		let out = expand(quote!(
			fn main(ui: &mut ::gneiss::ui::UiGuard) -> ::gneiss::ui::UiQueue<'_> {
				::gneiss::ui::UiQueue::new(ui)
			}
		));
		assert!(out.contains("UiGuard :: enter ()"), "{out}");
		assert!(out.contains("let queue = main (& mut ui)"), "{out}");
		assert!(out.contains("# [inline (never)] fn main"), "{out}");
		assert!(!out.contains("compile_error"), "{out}");
		let file: syn::File = syn::parse2(entry(
			quote!(),
			quote!(
				fn main(ui: &mut UiGuard) -> UiQueue<'_> {
					UiQueue::new(ui)
				}
			),
		))
		.unwrap();
		// prettyplease can't print the initializer-less `distributed_slice` statics, which syn
		// only parses as verbatim tokens
		for item in file.items {
			if let syn::Item::Verbatim(tokens) = item {
				println!("{tokens}");
			} else {
				println!("{}", prettyplease::unparse(&parse_quote!(#item)));
			}
		}
	}

	#[test]
	fn rejects_no_guard() {
		let out = expand(quote!(
			fn main() -> UiQueue<'static> {}
		));
		assert!(out.contains("takes a single"), "{out}");
	}

	#[test]
	fn rejects_unit_return() {
		let out = expand(quote!(
			fn main(ui: &mut UiGuard) {}
		));
		assert!(out.contains("returns the `UiQueue`"), "{out}");
	}
}
