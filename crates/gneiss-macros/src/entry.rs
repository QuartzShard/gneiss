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

	let mut out = item.clone();
	let input: Option<syn::ItemFn> = errs.handle(syn::parse2(item).map_err(darling::Error::from));

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
		// The guard drops before services register: their handlers can fire synchronously
		let call = if input.sig.inputs.is_empty() {
			quote! { #fn_name(); }
		} else {
			quote! {
				let mut ui = ::gneiss::ui::UiGuard::enter();
				#fn_name(&mut ui);
			}
		};

		out.extend(quote! {
			#[unsafe(export_name = "main")]
			#[doc = " Exported entry point, calls registration routines for `#[service]`s and other init chores"]
			pub extern "C" fn __gneiss_entry() {
				__gneiss_init();
                ::gneiss::log::debug!("gneiss init complete, starting event loop...");
				unsafe { ::gneiss::sys::app_event_loop() };
			}

			#[doc = " Setup, out of line so its stack frame is gone before the event loop runs"]
			#[inline(never)]
			fn __gneiss_init() {
                ::gneiss::log::debug!("Constructing UI...");

                ::gneiss::log::verbose!("If handlers fail to register next, check your main() returns.");
                ::gneiss::log::verbose!("Don't call `app_event_loop()`, that's handled by gneiss");
                // User code constructs UI and state
				{ #call }

                ::gneiss::log::debug!("Registering handlers...");
                // Handlers may run sync when hardware missing - require UI/State exists.
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
/// must be safe, non-async, non-variadic, non-generic, take no arguments or a
/// single `&mut UiGuard`, return `()`, and carry no symbol-export attributes.
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
	if sig.inputs.len() > 1 {
		errs.push(
			darling::Error::custom("entry fn takes no arguments, or a single `&mut UiGuard`")
				.with_span(&sig.inputs),
		);
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
	fn expands_main_with_ui() {
		let out = entry(
			quote!(),
			quote!(
				fn main(ui: &mut ::gneiss::ui::UiGuard) {}
			),
		);
		let out = out.to_string();
		assert!(out.contains("UiGuard :: enter ()"), "{out}");
		assert!(out.contains("main (& mut ui)"), "{out}");
	}

	#[test]
	fn expands_trivial_main() {
		let out = entry(
			quote!(),
			quote!(
				fn main() {}
			),
		);
		let file: syn::File = syn::parse2(out.clone()).unwrap_or_else(|e| panic!("{e}\n{out}"));
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
}
