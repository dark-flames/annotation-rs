use crate::Symbol;
use syn::MetaList;

pub trait Parse {
    fn get_path() -> Symbol;

    fn from_meta_list(input: &MetaList) -> Result<Self, syn::Error>
    where
        Self: std::marker::Sized;
}
