extern crate proc_macro;

use yui_test_attribute::attribute::{Simple, Full};
use yui::generate_reader;

generate_reader!(SimpleDerive, [Simple]);
generate_reader!(FullDerive, [Full]);