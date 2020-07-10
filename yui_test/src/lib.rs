extern crate proc_macro;

use yui::generate_reader;
use yui_test_attribute::attribute::{Full, Simple};

generate_reader!(SimpleDerive, [Simple]);
generate_reader!(FullDerive, [Full]);
