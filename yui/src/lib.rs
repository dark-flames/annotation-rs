#![feature(in_band_lifetimes)]

mod traits;
pub use traits::*;

pub use yui_internal::*;

#[doc(hidden)]
pub use yui_derive::{YuiAttribute, YuiEnumValue};

#[cfg(any(feature="generate-reader"))]
pub use yui_derive::{generate_reader, __get_attribute, __has_attribute};

#[cfg(any(feature="generate-reader"))]
#[macro_export]
macro_rules! get_attribute {
    ($class: ident, $attr:ident) => {
        $crate::__get_attribute!($class, $attr)
    };
    ($class: ident :: $prop: ident, $attr:ident) => {
        $crate::__get_attribute!($class, $attr, $prop)
    };
}

#[cfg(any(feature="generate-reader"))]
#[macro_export]
macro_rules! has_attribute {
    ($class: ident :: $prop: ident, $attr:ident) => {
        $crate::__has_attribute!($class, $attr, $prop)
    };
    ($class: ident, $attr:ident) => {
        crate::__has_attribute!($class, $attr)
    };
}
