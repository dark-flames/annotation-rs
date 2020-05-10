use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub struct Error {
    pub message: &'static str,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error { message }
    }

    pub fn from_syn_error(error: syn::Error) -> Self {
        Self::new(error.to_string().as_str())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(self.message)
    }
}
