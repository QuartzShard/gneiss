use darling::Error as DarlingError;
use quote::{format_ident, quote};
use syn::{
	Attribute, Expr, Ident, ItemStruct, Meta, Path, Token, Type, TypeSlice, Visibility,
	parse::{Parse, ParseStream, Parser},
	punctuated::Punctuated,
	spanned::Spanned,
};

/// One `persist_keys!` entry: attributes, visibility, name, then what it holds.
struct Entry {
	attrs: Vec<Attribute>,
	vis:   Visibility,
	name:  Ident,
	kind:  Kind,
}

enum Kind {
	/// `NAME: T`, one key
	Key(Type),
	/// `NAME: [T; len]`, `len` keys
	Block(Type, Expr),
	/// `NAME: [T; ..]`, every key after the rest
	Tail(Type),
}

impl Parse for Entry {
	fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
		let attrs = input.call(Attribute::parse_outer)?;
		let vis = input.parse()?;
		let name = input.parse()?;
		input.parse::<Token![:]>()?;
		let kind = if input.peek(syn::token::Bracket) {
			let content;
			let bracket_token = syn::bracketed!(content in input);
			let elem: Type = content.parse()?;
			if content.is_empty() {
				// `[T]` is a slice key, not a block
				Kind::Key(Type::Slice(TypeSlice {
					attrs: Vec::new(),
					bracket_token,
					elem: Box::new(elem),
				}))
			} else {
				content.parse::<Token![;]>()?;
				if content.peek(Token![..]) {
					content.parse::<Token![..]>()?;
					Kind::Tail(elem)
				} else {
					Kind::Block(elem, content.parse()?)
				}
			}
		} else {
			Kind::Key(input.parse()?)
		};
		Ok(Self {
			attrs,
			vis,
			name,
			kind,
		})
	}
}

pub fn persist_keys(input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
	let mut errs = DarlingError::accumulator();
	let mut out = proc_macro2::TokenStream::new();

	let entries = errs.handle(
		Punctuated::<Entry, Token![,]>::parse_terminated
			.parse2(input)
			.map_err(DarlingError::from),
	);
	// The first key after the fixed entries, where a `[T; ..]` tail starts
	let mut end = quote!(0u32);
	if let Some(entries) = entries {
		let last = entries.len().saturating_sub(1);
		// Each entry's first key goes in its own hidden const: the previous const plus the previous
		// entry's length.
		let mut previous: Option<(Ident, proc_macro2::TokenStream)> = None;
		for (i, entry) in entries.iter().enumerate() {
			let first = format_ident!("__GNEISS_PERSIST_KEY_{}", i);
			let start = match &previous {
				None => quote!(0u32),
				Some((key, len)) => quote!(#key + #len),
			};
			out.extend(quote! {
				#[doc(hidden)]
				#[allow(dead_code)]
				const #first: u32 = #start;
			});

			let Entry {
				attrs,
				vis,
				name,
				kind,
			} = entry;
			let (ty, init, len) = match kind {
				Kind::Key(ty) => (
					quote!(::gneiss::resource::persist::PersistKey<#ty>),
					quote!(::gneiss::resource::persist::PersistKey::__new(#first)),
					quote!(1u32),
				),
				Kind::Block(ty, len) => (
					quote!(::gneiss::resource::persist::PersistBlock<#ty>),
					quote!(::gneiss::resource::persist::PersistBlock::__new(#first, #len)),
					quote!((#len)),
				),
				Kind::Tail(ty) => {
					if i != last {
						errs.push(
							DarlingError::custom(
								"`[T; ..]` takes every key after it, so must be last",
							)
							.with_span(&name),
						);
					}
					(
						quote!(::gneiss::resource::persist::PersistTail<#ty>),
						quote!(::gneiss::resource::persist::PersistTail::__new(#first)),
						quote!(0u32),
					)
				}
			};
			out.extend(quote! {
				#(#attrs)*
				#vis const #name: #ty = #init;
			});
			previous = Some((first, len));
		}
		if let Some((key, len)) = previous {
			end = quote!(#key + #len);
		}
	}

	// Enforce one-per-app, also stash end count. Killed by LTO
	out.extend(quote! {
		const _: () = {
			#[unsafe(export_name = "__gneiss_persist_keys")]
			static GUARD: u32 = #end;
		};
	});

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}

pub fn make_persistible(
	attr: proc_macro2::TokenStream,
	item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
	let mut errs = DarlingError::accumulator();
	let mut out = item.clone();

	if !attr.is_empty() {
		errs.push(DarlingError::custom("make_persistible takes no arguments").with_span(&attr));
	}
	let item: Option<ItemStruct> = errs.handle(syn::parse2(item).map_err(|e| {
		DarlingError::custom("make_persistible applies to a struct").with_span(&e.span())
	}));

	if let Some(item) = item {
		// Only transform a struct that passes: otherwise the derives' errors bury the reason
		let mut valid = true;
		if !item.generics.params.is_empty() {
			valid = false;
			errs.push(
				DarlingError::custom("make_persistible doesn't support generic structs")
					.with_span(&item.generics),
			);
		}

		// What the struct already derives, by last path segment, so they aren't derived twice
		let mut existing = Vec::new();
		let mut has_repr = false;
		for attr in &item.attrs {
			if attr.path().is_ident("derive") {
				let paths = errs.handle(
					attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
						.map_err(DarlingError::from),
				);
				valid &= paths.is_some();
				existing.extend(
					paths
						.iter()
						.flatten()
						.filter_map(|p| p.segments.last().map(|s| s.ident.to_string())),
				);
			} else if attr.path().is_ident("repr") {
				let reprs = errs.handle(
					attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
						.map_err(DarlingError::from),
				);
				let fits = reprs
					.iter()
					.flatten()
					.all(|r| r.path().is_ident("C") || r.path().is_ident("transparent"));
				valid &= reprs.is_some() && fits;
				if !fits {
					errs.push(
						DarlingError::custom("make_persistible needs repr(C) or repr(transparent)")
							.with_span(attr),
					);
				}
				has_repr = true;
			}
		}

		if valid {
			let wanted = [
				("Clone", quote!(::core::clone::Clone)),
				("Copy", quote!(::core::marker::Copy)),
				("Pod", quote!(::gneiss::bytemuck::Pod)),
				("Zeroable", quote!(::gneiss::bytemuck::Zeroable)),
			];
			let derives = wanted
				.iter()
				.filter(|(name, _)| !existing.iter().any(|d| d == name))
				.map(|(_, path)| path);
			let repr = (!has_repr).then(|| quote!(#[repr(C)]));
			let name = &item.ident;

			out = quote! {
				#[derive(#(#derives),*)]
				#[bytemuck(crate = "::gneiss::bytemuck")]
				#repr
				#item

				const _: () = ::core::assert!(
					::core::mem::size_of::<#name>() <= ::gneiss::sys::PERSIST_DATA_MAX_LENGTH as usize,
					"a make_persistible struct must be at most PERSIST_DATA_MAX_LENGTH (256) bytes",
				);

				impl ::gneiss::resource::persist::Persist for #name {

					fn persist(
						&self,
						key: ::gneiss::resource::persist::PersistKey<Self>,
					) -> ::core::result::Result<u32, ::gneiss::status::StatusError> {
						::gneiss::resource::persist::__persist_pod(self, key.key())
					}
				}

				impl ::gneiss::resource::persist::Recover for #name {
					fn recover(
					key: ::gneiss::resource::persist::PersistKey<Self>,
				) -> ::core::option::Option<Self> {
						::gneiss::resource::persist::__recover_pod(key.key())
					}
				}
			};
		}
	}

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}
