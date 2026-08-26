
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(EnumFromBackingType, attributes(default))]
pub fn derive_enum_from_backing_type(tokens: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(tokens as DeriveInput);

	let enum_name = ast.ident;

	let backing_type = {
		let attribute = ast.attrs.iter()
			.filter_map(|a| a.meta.require_list().map(|a| Some(a)).unwrap_or(None))
			.find(|ml| ml.path.is_ident("repr"));
		if attribute.is_none() { todo!() }
		attribute.unwrap().tokens.clone().into_iter().next().unwrap()
	};

	let enum_data = {
		let data = ast.data;
		if let Data::Enum(enum_data) = data { enum_data }
		else { todo!() }
	};

	let mut enum_vars = TokenStream2::new();
	enum_vars.extend(
		enum_data.variants.iter()
			.map(|v| (v.ident.clone(), v.discriminant.clone().unwrap().1))
			.map(|(ident, value)| {
				let tokens = TokenStream2::from(TokenStream::from(quote! {#value => #enum_name::#ident,}));
				println!("{:?}", tokens);
				tokens
			})
	);

	let default_variant = &enum_data.variants.iter()
		.find(|v| v.attrs.iter().find(
			|a| a.path().is_ident("default")
		).is_some())
		.unwrap()
		.ident;

	let expanded = quote! {
		impl From<#backing_type> for #enum_name {
			fn from(value: #backing_type) -> #enum_name {
				match value {
					#enum_vars
					_ => #enum_name::#default_variant,
				}
			}
		}
	};

	TokenStream::from(expanded)
}
