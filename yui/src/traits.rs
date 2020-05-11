use crate::{get_lit_str, Symbol};
use std::str::FromStr;
use syn::{Error, Lit, MetaList, Path};

pub trait ValueEnum: FromStr {
    fn from_str(s: &str) -> Result<Self, Self::Err>;

    fn read_from_lit(lit: &Lit, path: &Path) -> Result<Self, Error> {
        match get_lit_str(lit, path)?.parse() {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::new_spanned(
                lit,
                "Some error occurred while parse lit into enum",
            )),
        }
    }
}

pub trait Parse {
    fn get_path() -> Symbol;

    fn from_meta_list(input: &MetaList) -> Result<Self, Error>
    where
        Self: std::marker::Sized;
}
