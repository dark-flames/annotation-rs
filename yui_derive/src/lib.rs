#![feature(in_band_lifetimes)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod field;
mod ty;

mod attribute;
use attribute::Attribute;

mod enum_value;
use enum_value::EnumValue;

#[proc_macro_derive(YuiEnumValue, attributes(variant_value))]
pub fn derive_enum_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_value = EnumValue::from_ast(&input);
    TokenStream::from(match enum_value {
        Ok(value) => value.get_lit_reader(),
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro_derive(YuiAttribute, attributes(attribute, attribute_field))]
pub fn derive_attribute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let attribute = Attribute::from_ast(&input);

    TokenStream::from(match attribute {
        Ok(value) => value.get_reader(),
        Err(e) => e.to_compile_error(),
    })
}
