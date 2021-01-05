extern crate proc_macro;

use derive::generate_reader;
use objects::attributes::{Full, Simple};

generate_reader!(SimpleDerive, [Simple]);
generate_reader!(FullDerive, [Full]);
