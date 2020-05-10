use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result};
use syn::token::Token;
use syn::Error as SynError;

#[derive(Debug, Clone, Copy)]
pub struct Error {
    pub message: &'static str,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error { message }
    }

    pub fn from_syn_error<T: Token, D: Display>(token: T, message: D) -> Self {
        Self::new(SynError::new_spanned(token, message).to_string().as_str())
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
