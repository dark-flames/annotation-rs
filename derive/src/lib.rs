extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod field;
mod ty;

mod attribute;
use attribute::Annotation;

mod enum_value;
use crate::reader::{GetAnnotationParam, ReaderConfig};
use enum_value::EnumValue;

mod reader;

#[proc_macro_derive(AnnotationEnumValue, attributes(variant_value, mod_path))]
pub fn derive_enum_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_value = EnumValue::from_ast(&input);

    TokenStream::from(match enum_value {
        Ok(value) => value.get_implement(),
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro_derive(Annotation, attributes(field, mod_path))]
pub fn derive_attribute(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let attribute = Annotation::from_ast(&input);

    TokenStream::from(match attribute {
        Ok(value) => value.get_implement(),
        Err(e) => e.to_compile_error(),
    })
}

#[proc_macro]
pub fn generate_reader(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as ReaderConfig);
    TokenStream::from(config.get_reader())
}

#[proc_macro]
pub fn __get_annotation(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as GetAnnotationParam);
    TokenStream::from(config.get_annotation())
}

#[proc_macro]
pub fn __has_annotation(input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(input as GetAnnotationParam);
    TokenStream::from(config.has_annotation())
}
