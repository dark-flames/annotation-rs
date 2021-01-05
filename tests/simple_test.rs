#![feature(proc_macro_hygiene)]
#[cfg(any(feature = "generate-reader"))]
use annotation_rs::get_attribute;

use float_cmp::approx_eq;
use test_macro::SimpleDerive;
use test_object::attributes::Simple;
use test_object::enums::TestEnum;

#[cfg(any(feature = "generate-reader"))]
#[derive(SimpleDerive)]
#[Simple(i32 = 1, u16 = 2, float = 1.1, string = "test", enum2 = "aaa")]
struct Test;

#[cfg(any(feature = "generate-reader"))]
#[test]
pub fn simple_test() {
    let attr: Simple = get_attribute!(Test, Simple).unwrap();
    assert_eq!(attr.int32, 1);
    assert_eq!(attr.unsigned16, 2);
    let float = attr.float;
    assert!(approx_eq!(f32, float, 1.1));
    assert_eq!(attr.string, "test");
    assert_eq!(attr.enum1, Some(TestEnum::VariantC));
    assert_eq!(attr.enum2, TestEnum::VariantA);
}
