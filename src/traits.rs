use crate::{get_lit_str, Error, Symbol};
use std::str::FromStr;
use syn::{Attribute, Lit, Path};

pub trait ValueEnum: FromStr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err>;

    fn read_from_lit(lit: &Lit, path: &Path) -> Result<Self, Self::Err> {
        Self::Err::get_lit_str(lit, path)?.parse()
    }
}

pub trait Parse {
    fn get_path() -> Symbol {}
    fn from_token_stream(input: &Attribute) -> Result<Self, Error>;
}
