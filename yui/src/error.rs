use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
}
impl Error {
    pub fn new<T: Display>(message: T) -> Self {
        Error {
            message: message.to_string(),
        }
    }
    pub fn get_message(&self) -> String {
        self.message.clone()
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.message)
    }
}
