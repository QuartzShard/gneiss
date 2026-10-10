use darling::Error as DarlingError;
use quote::{format_ident, quote, quote_spanned};
use syn::{Expr, ItemFn, Path, Token, parse::Parser, punctuated::Punctuated, spanned::Spanned};

#[derive(Debug, Clone)]
struct ServiceArgs {
	service: Path,
	config:  Punctuated<Expr, Token![,]>,
}

impl syn::parse::Parse for ServiceArgs {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let service: Path = input.parse()?;
		let config = if input.peek(Token![,]) {
			input.parse::<Token![,]>()?;
			Punctuated::parse_terminated(input)?
		} else {
			Punctuated::new()
		};
		Ok(Self { service, config })
	}
}

pub fn service(
	attr: proc_macro2::TokenStream,
	item: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
	let mut errs = DarlingError::accumulator();
	let mut out = item.clone();

	let fn_item: Option<ItemFn> = errs.handle(syn::parse2(item).map_err(DarlingError::from));
	let attr_args: Option<ServiceArgs> = errs.handle(syn::parse2(attr).map_err(DarlingError::from));
	let mut service: Option<Path> = attr_args.clone().map(|a| a.service);
	let mut config: Vec<Expr> =
		attr_args.map_or_else(Vec::new, |a| a.config.iter().cloned().collect());

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
		let cfg_body = if config.is_empty() {
			quote! { ::core::default::Default::default() }
		} else {
			quote! {( #(#config),* )}
		};

		let handler_check = quote_spanned! { fn_ident.span() =>
			const _: <#service as ::gneiss::service::Service>::Handler = #fn_ident;
			const _: <#service as ::gneiss::service::Service>::RawHandler = #raw_fn_ident;
		};
		let config_fn = quote_spanned! { cfg_span =>
			fn #cfg_fn_ident() -> <#service as ::gneiss::service::Service>::SubscribeArg {
				#cfg_body
			}
		};

		out.extend(quote! {
			#handler_check
			#config_fn
			::gneiss::#m!(handler: #fn_ident, raw_handler: #raw_fn_ident, service: #service);
			#[::gneiss::linkme::distributed_slice(crate::GNEISS_SERVICES)]
			#[linkme(crate = ::gneiss::linkme)]
			fn #init_fn_ident() {
				<#service as ::gneiss::service::Service>::subscribe(#raw_fn_ident, #cfg_fn_ident())
			}
		});
	}

	if let Err(e) = errs.finish() {
		out.extend(e.write_errors());
	}
	out
}
