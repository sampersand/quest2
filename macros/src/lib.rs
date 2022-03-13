use proc_macro2::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(NamedType)]
pub fn named_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	expand_named_type(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

fn expand_named_type(input: DeriveInput) -> syn::Result<TokenStream> {
	let name = input.ident;

	Ok(quote! {
		impl crate::value::NamedType for crate::value::Gc<#name> {
			const TYPENAME: &'static str = stringify!(#name);
		}
	})
}
