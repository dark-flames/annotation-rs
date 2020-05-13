#![feature(in_band_lifetimes)]

mod helper;
pub use helper::{get_lit_bool, get_lit_float, get_lit_int, get_lit_str, unwrap_punctuated_first};

pub mod symbol;
pub use crate::symbol::Symbol;

mod traits;
pub use crate::traits::Parse;

mod enum_value;
pub use crate::enum_value::{EnumItem, EnumValue};

mod ty;
pub use crate::ty::{DefaultValue, FieldType, Type};

mod field;
pub use crate::field::{Fields, NamedField, UnnamedFiled};

mod attribute;
pub use crate::attribute::Attribute;

mod error;
pub use crate::error::Error;
