use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_quote;

pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}
