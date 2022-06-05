use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::{Data, DataStruct, Fields};

pub fn expand_allocated(input: DeriveInput) -> TokenStream {
	let name = input.ident;

	// quote! {
	// 	impl ::quest::value::NamedType for ::quest::value::Gc<#name> {
	// 		const TYPENAME: ::quest::value::Typename = stringify!(#name);
	// 	}
	// }

	let fields = match input.data {
		Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
		_ => panic!("this derive macro only works on structs with named fields"),
	};

	let internal_name = format!("{name}Internal");

	// let fields = fields.into_iter().map(|Field { ident, ty, .. } | {
	// 	// Interpolation only works for variables, not arbitrary expressions.
	// 	// That's why we need to move these fields into local variables first
	// 	// (borrowing would also work though).
	// 	let field_name = f.ident;
	// 	let field_ty = f.ty;

	// 	quote! {
	// 		#ident: #field_ty
	// 	}
	// });
	let fields = fields.into_iter();

	quote! {
		pub struct #name(crate::value::base::Base<#internal_name>);

		#[doc(hidden)]
		#[automatically_derived]
		pub struct #internal_name {
			#(#fields)*
		}

		#[allow(unsafe_code)]
		unsafe impl crate::value::gc::Allocated for #name {
			type Inner = #internal_name;
		}
	}
}

/*

#[proc_macro_derive(Allocated)]
pub fn allocated(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	expand_allocated(input).into()
}

fn expand_allocated(input: DeriveInput) -> TokenStream {
	let name = input.ident;

	quote! {
		impl quest::value::NamedType for crate::value::Gc<#name> {
			const TYPENAME: crate::value::Typename = stringify!(#name);
		}
	}
}

/*

#[macro_export]
macro_rules! quest_type {
	(
		$(#[$meta:meta])*
		$vis:vis struct $name:ident $(<$($gen:ident),*>)? $({$innervis:vis})? ($($inner:tt)*) $(where {$($cond:tt)*})?;
	) => {
		$(#[$meta])*
		#[repr(transparent)]
		$vis struct $name $(<$($gen)*>)?($($innervis)? $crate::value::base::Base<$($inner)*>) $(where $($cond)*)?;

		unsafe impl $(<$($gen),*>)? $crate::value::gc::Allocated for $name $(<$($gen),*>)?
		$(where $($cond)*)? {
			type Inner = $($inner)*;
		}
	};
}
*/
*/
