#![feature(in_band_lifetimes)]

mod traits;
pub use traits::*;

extern crate yui_internal;
pub use yui_internal::*;

#[cfg(feature = "yui_derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate yui_derive;

#[cfg(feature = "yui_derive")]
#[doc(hidden)]
pub use yui_derive::{YuiAttribute, YuiEnumValue, generate_reader, __get_attribute, __has_attribute};

#[macro_export]
macro_rules! get_attribute {
    ($class: ident, $attr:ident) => {
        yui::__get_attribute!($class, $attr)
    };
    ($class: ident :: $prop: ident, $attr:ident) => {
        yui::__get_attribute!($class, $attr, $prop)
    };
}

#[macro_export]
macro_rules! has_attribute {
    ($class: ident :: $prop: ident, $attr:ident) => {
        yui::__has_attribute!($class, $attr, $prop)
    };
    ($class: ident, $attr:ident) => {
        yui::__has_attribute!($class, $attr)
    };
}
