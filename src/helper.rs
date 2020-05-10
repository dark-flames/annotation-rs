use super::error::Error;
use syn::{Error as SynError, Lit, Path};

pub fn get_lit_str(lit: &Lit, path: &Path) -> Result<String, Error> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.value()),
        _ => Err(Error::from_syn_error(SynError::new_spanned(
            lit,
            format!("expected {} attribute to be a string", path),
        ))),
    }
}
