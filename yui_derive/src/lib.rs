use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use yui::EnumValue;

#[proc_macro_derive(EnumValue, attributes(enum_item_value))]
pub fn derive_enum_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_value = EnumValue::from_ast(&input);

    match enum_value {
        Ok(value) => TokenStream::from(value.get_lit_reader()),
        Err(e) => {
            println!("{}", e);
            panic!();
        }
    }
}
