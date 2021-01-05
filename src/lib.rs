mod traits;
pub use traits::*;

pub use helpers::*;

#[doc(hidden)]
pub use derive::{Annotation, AnnotationEnumValue};

#[cfg(any(feature = "annotation_reader"))]
pub use derive::{__get_annotation, __has_annotation, generate_reader};

#[cfg(any(feature = "annotation_reader"))]
#[macro_export]
macro_rules! get_annotation {
    ($class: ident, $annotation: ident) => {
        $crate::__get_annotation!($class, $annotation)
    };
    ($class: ident :: $prop: ident, $annotation: ident) => {
        $crate::__get_annotation!($class, $annotation, $prop)
    };
}

#[cfg(any(feature = "annotation_reader"))]
#[macro_export]
macro_rules! has_annotation {
    ($class: ident :: $prop: ident, $annotation: ident) => {
        $crate::__has_annotation!($class, $annotation, $prop)
    };
    ($class: ident, $annotation: ident) => {
        $crate::__has_annotation!($class, $annotation)
    };
}
