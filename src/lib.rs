mod traits;
pub use traits::*;

pub use helpers::*;

#[doc(hidden)]
pub use derive::{Annotation, AnnotationEnumValue};

#[cfg(any(feature = "generate-reader"))]
pub use derive::{__get_attribute, __has_attribute, generate_reader};

#[cfg(any(feature = "generate-reader"))]
#[macro_export]
macro_rules! get_attribute {
    ($class: ident, $attr: ident) => {
        $crate::__get_attribute!($class, $attr)
    };
    ($class: ident :: $prop: ident, $attr: ident) => {
        $crate::__get_attribute!($class, $attr, $prop)
    };
}

#[cfg(any(feature = "generate-reader"))]
#[macro_export]
macro_rules! has_attribute {
    ($class: ident :: $prop: ident, $attr: ident) => {
        $crate::__has_attribute!($class, $attr, $prop)
    };
    ($class: ident, $attr: ident) => {
        $crate::__has_attribute!($class, $attr)
    };
}
