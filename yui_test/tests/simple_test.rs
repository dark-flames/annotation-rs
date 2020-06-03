#![feature(proc_macro_hygiene)]
use yui::get_attribute;
use yui_test::SimpleDerive;
use yui_test_attribute::attribute::Simple;
use yui_test_attribute::enums::TestEnum;

#[derive(SimpleDerive)]
#[Simple(i32 = 1, u16 = 2, float = 1.1, string = "test", enum2 = "aaa")]
struct Test;

#[test]
pub fn simple_test() {
    let attr: Simple = get_attribute!(Test, Simple).unwrap();
    assert_eq!(attr.int32, 1);
    assert_eq!(attr.unsigned16, 2);
    assert_eq!(attr.float, 1.1);
    assert_eq!(attr.string, "test");
    assert_eq!(attr.enum1, Some(TestEnum::VariantC));
    assert_eq!(attr.enum2, TestEnum::VariantA);
}
