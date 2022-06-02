use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(NamedType)]
pub fn named_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	expand_named_type(input).into()
}

fn expand_named_type(input: DeriveInput) -> TokenStream {
	let name = input.ident;

	quote! {
		impl crate::value::NamedType for crate::value::Gc<#name> {
			const TYPENAME: crate::value::Typename = stringify!(#name);
		}
	}
}
