use darling::Error as DarlingError;
use quote::quote;
use syn::{Ident, ImplItem, ItemImpl, spanned::Spanned};

/// Methods that must be written together: the firmware calls the draw for whatever the height gives
const PAIRS: [(&str, &str); 2] = [
	("get_header_height", "draw_header"),
	("get_separator_height", "draw_separator"),
];

pub fn menu(
	attr: proc_macro2::TokenStream,
	item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
	let mut errs = DarlingError::accumulator();
	let mut out = item.clone();

	if !attr.is_empty() {
		errs.push(DarlingError::custom("menu takes no arguments").with_span(&attr));
	}
	let item: Option<ItemImpl> = errs.handle(syn::parse2(item).map_err(|e| {
		DarlingError::custom("menu applies to an `impl Menu for` block").with_span(&e.span())
	}));

	if let Some(item) = item {
		if item.trait_.is_none() {
			errs.push(
				DarlingError::custom("menu applies to an `impl Menu for` block")
					.with_span(&item.self_ty),
			);
		} else {
			// Every `Menu` method is a callback, named after its `MenuLayerCallbacks` field
			let written: Vec<Ident> = item
				.items
				.iter()
				.filter_map(|i| match i {
					ImplItem::Fn(f) => Some(f.sig.ident.clone()),
					_ => None,
				})
				.collect();
			let span_of = |name: &str| written.iter().find(|w| *w == name).map(Ident::span);

			for (height, draw) in PAIRS {
				let (written, missing) = match (span_of(height), span_of(draw)) {
					(Some(span), None) => ((height, span), draw),
					(None, Some(span)) => ((draw, span), height),
					_ => continue,
				};
				errs.push(
					DarlingError::custom(format!(
						"`{}` needs `{missing}` too: they're one callback pair",
						written.0
					))
					.with_span(&written.1),
				);
			}

			// Written even alongside a pair error, which already fails the build: without it, a
			// second error would blame the missing attribute
			let (impl_generics, _, where_clause) = item.generics.split_for_impl();
			let self_ty = &item.self_ty;
			out.extend(quote! {
				impl #impl_generics ::gneiss::ui::__menu::MenuCallbacks for #self_ty #where_clause {
					const CALLBACKS: ::gneiss::sys::MenuLayerCallbacks =
						::gneiss::sys::MenuLayerCallbacks {
							#(#written: ::core::option::Option::Some(::gneiss::ui::__menu::#written::<Self>),)*
							..::gneiss::ui::__menu::NONE
						};
				}
			});
		}
	}

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}

#[cfg(test)]
mod tests {
	use quote::quote;

	use super::menu;

	fn expand(item: proc_macro2::TokenStream) -> String {
		menu(quote!(), item).to_string()
	}

	#[test]
	fn sets_only_written_callbacks() {
		let out = expand(quote! {
			impl Menu for Settings {
				type Data = Vec<u8>;
				fn get_num_rows(data: &Self::Data, _: u16) -> u16 { 0 }
				fn draw_row(data: &Self::Data, cell: &MenuCanvas<'_>, index: MenuIndex, ctx: &mut GContext<'_>) {}
				fn select_click(data: &mut Self::Data, index: MenuIndex) {}
			}
		});
		for set in ["get_num_rows", "draw_row", "select_click"] {
			assert!(
				out.contains(&format!("{set} : :: core :: option :: Option :: Some")),
				"{out}"
			);
		}
		for unset in ["get_num_sections", "get_cell_height", "select_long_click"] {
			assert!(!out.contains(unset), "{out}");
		}
		assert!(
			out.contains(".. :: gneiss :: ui :: __menu :: NONE"),
			"{out}"
		);
	}

	#[test]
	fn rejects_half_a_pair() {
		let out = expand(quote! {
			impl Menu for Settings {
				type Data = ();
				fn get_header_height(data: &Self::Data, section: u16) -> i16 { 16 }
			}
		});
		assert!(out.contains("needs `draw_header` too"), "{out}");
		assert!(out.contains("MenuCallbacks"), "{out}");
	}

	#[test]
	fn rejects_inherent_impl() {
		let out = expand(quote! {
			impl Settings {}
		});
		assert!(out.contains("compile_error"), "{out}");
	}
}
