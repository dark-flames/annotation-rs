#![feature(in_band_lifetimes)]

mod traits;
pub use crate::traits::AttributeStructure;

mod helper;
pub use helper::*;

pub mod symbol;
pub use crate::symbol::Symbol;

mod error;
pub use crate::error::Error;
