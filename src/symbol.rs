use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

pub const ATTRIBUTE: Symbol = Symbol("attribute");
pub const ATTRIBUTE_FIELD: Symbol = Symbol("attribute_field");
pub const ATTRIBUTE_FIELD_PATH: Symbol = Symbol("path");
pub const ATTRIBUTE_FIELD_ENUM_VALUE: Symbol = Symbol("enum_value");
pub const ATTRIBUTE_FIELD_DEFAULT: Symbol = Symbol("default");
pub const ENUM_ITEM_VALUE: Symbol = Symbol("enum_item_value");
