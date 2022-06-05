use syn::{parse_macro_input, DeriveInput};
mod allocated;
mod named_type;

#[proc_macro_derive(NamedType)]
pub fn named_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	named_type::expand_named_type(input).into()
}

#[proc_macro_derive(Allocated)]
pub fn allocated(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	allocated::expand_allocated(input).into()
}
