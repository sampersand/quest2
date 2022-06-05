use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn expand_named_type(input: DeriveInput) -> TokenStream {
	let name = input.ident;

	quote! {
		impl quest::value::NamedType for quest::value::Gc<#name> {
			const TYPENAME: quest::value::Typename = stringify!(#name);
		}
	}
}
