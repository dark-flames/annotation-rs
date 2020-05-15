use crate::Symbol;
use syn::Meta;

pub trait AttributeStructure {
    fn get_path() -> Symbol;

    fn from_meta_list(input: &Meta) -> Result<Self, syn::Error>
    where
        Self: std::marker::Sized;
}
