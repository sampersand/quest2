#![allow(unused)]
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::ItemFn;
use syn::MetaNameValue;
use syn::NestedMeta;

fn get_qs_name(args: &[NestedMeta]) -> Option<String> {
	for arg in args {
		match arg {
			NestedMeta::Meta(syn::Meta::NameValue(MetaNameValue {
				path: syn::Path { leading_colon: None, segments },
				lit: syn::Lit::Str(litstr),
				..
			})) if segments.len() == 1 && segments[0].ident == "name" => {
				let mut s = litstr.token().to_string();
				s.pop();
				s.remove(0);
				return Some(s);
			}
			_ => {}
		}
	}
	None
}

pub fn expand_quest_fn(args: syn::AttributeArgs, item: ItemFn) -> TokenStream {
	let vis = &item.vis;
	let name = &item.sig.ident;
	let qs_name = format_ident!("qs_{}", get_qs_name(&args).unwrap_or_else(|| name.to_string()));

	let positional_len = item.sig.inputs.len() - 1;

	let attrs = item.sig.inputs.iter().skip(1).enumerate().map(|(i, x)| {
		let kind = match x {
			syn::FnArg::Typed(syn::PatType { ref ty, .. }) => ty,
			_ => unreachable!(),
		};
		quote! {
			args[#i].try_downcast::<#kind>()?
		}
	});

	// let (returns_result, to_value) = match item.sig.output {
	// 	syn::ReturnType::Type(_, ref foo)
	// 		if matches!(&**foo,
	// 		syn::Type::Path(syn::TypePath {
	// 			qself: None,
	// 			path: syn::Path { leading_colon: None, segments },
	// 		}) if segments.len() == 1 && segments[0].ident == "Result") =>
	// 	{
	// 		false
	// 	}
	// 	_ => true,
	// };

	// let ret = if to_value {
	// 	quote! {
	// 		Ok(result)
	// 	}
	// } else {
	// 	quote! {
	// 		result
	// 	}
	// }

	let qs_fn = quote! {
		#vis fn #qs_name (
			object: ::quest::Value,
			args: ::quest::vm::Args<'_>
		) -> ::quest::Result<::quest::Value>
		{
			args.assert_no_keyword()?;
			args.assert_positional_len(#positional_len)?;

			let this = object.try_downcast::<Self>()?;
			let result = this.#name(#(#attrs,)*);
			result
		}
	};

	let mut ts = TokenStream::new();

	ts.extend(item.into_token_stream());
	ts.extend(qs_fn.into_token_stream());
	ts
}
