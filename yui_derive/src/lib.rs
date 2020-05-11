use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use yui::EnumValue;

#[proc_macro_derive(EnumValue, attributes(enum_item_value))]
pub fn derive_enum_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_value = EnumValue::from_ast(&input);

    //panic!(enum_value.unwrap().get_lit_reader().to_string());

    TokenStream::from(match enum_value {
        Ok(value) => value.get_lit_reader(),
        Err(e) => e.to_compile_error(),
    })
}
