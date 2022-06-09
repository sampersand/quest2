use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemFn};
mod allocated;
mod named_type;
mod quest_fn;

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

#[proc_macro_attribute]
pub fn quest_fn(
	attr: proc_macro::TokenStream,
	item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	let args = parse_macro_input!(attr as AttributeArgs);
	let item = parse_macro_input!(item as ItemFn);

	quest_fn::expand_quest_fn(args, item).into()
}
