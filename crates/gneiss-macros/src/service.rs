use darling::Error as DarlingError;
use quote::{format_ident, quote, quote_spanned};
use syn::{Expr, ItemFn, Path, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};

pub fn service(
	attr: proc_macro2::TokenStream,
	item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
	let mut errs = DarlingError::accumulator();
	let mut out = item.clone();

	let fn_item: Option<ItemFn> = errs.handle(syn::parse2(item).map_err(DarlingError::from));
	let attr_args: Option<Punctuated<Expr, Token![,]>> = errs.handle(
		Punctuated::<Expr, Token![,]>::parse_terminated
			.parse2(attr)
			.map_err(DarlingError::from),
	);
	let mut service: Option<Path> = None;
	let mut config: Vec<Expr> = Vec::new();
	if let Some(attr_args) = attr_args {
		let mut it = attr_args.into_iter();
		match it.next() {
			Some(Expr::Path(p)) => service = Some(p.path),
			Some(other) => {
				errs.push(DarlingError::custom("expected a service type").with_span(&other));
			}
			None => errs.push(DarlingError::custom("expected #[service(SomeService, ..)]")),
		}
		config = it.collect();
	}

	if let (Some(fn_item), Some(service)) = (fn_item, service) {
		let fn_ident = fn_item.sig.ident;
		let cfg_span = config.first().map_or_else(|| service.span(), Spanned::span);

		let seg = &service
			.segments
			.last()
			.expect("a path has >=1 segment")
			.ident;
		let m = format_ident!("{seg}_raw");
		let raw_fn_ident = format_ident!("__{}_raw", fn_ident);
		let init_fn_ident = format_ident!("__gneiss_service_{}_init", fn_ident);
		let cfg_fn_ident = format_ident!("__gneiss_service_{}_cfg", fn_ident);
		let static_ident = format_ident!("{}_ENTRY", fn_ident.to_string().to_uppercase());

		let handler_check = quote_spanned! { fn_ident.span() =>
			const _: <#service as ::gneiss::service::Service>::Handler = #fn_ident;
			const _: <#service as ::gneiss::service::Service>::RawHandler = #raw_fn_ident;
		};
		let config_fn = quote_spanned! { cfg_span =>
			fn #cfg_fn_ident() -> <#service as ::gneiss::service::Service>::SubscribeArg {
				( #(#config),* )
			}
		};

		out.extend(quote! {
			#handler_check
			#config_fn
			::gneiss::#m!(handler: #fn_ident, raw_handler: #raw_fn_ident);
			fn #init_fn_ident() {
				<#service as ::gneiss::service::Service>::subscribe(#raw_fn_ident, #cfg_fn_ident())
			}
			#[::gneiss::linkme::distributed_slice(GNEISS_SERVICES)]
			#[linkme(crate = ::gneiss::linkme)]
			static #static_ident: fn() = #init_fn_ident;
		});
	}

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}
