use super::EnumValue;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(EnumValue, attributes(enum_value_item))]
pub fn derive_enum_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input(input as DeriveInput);

    let enum_value = EnumValue::from_ast(input).unwrap_or_else(handle_derive_error);

    enum_value.get_lit_reader()
}

pub fn handle_derive_error(error: syn::Error) -> proc_macro2::TokenStream {
    let compile_errors = error.to_compile_error();
    quote!(#(#compile_errors)*)
}
