#![feature(proc_macro_hygiene)]
use yui::get_attribute;
use yui_test::TestDerive;
use yui_test_attribute::attribute::Simple;
use yui_test_attribute::enums::TestEnum;

#[derive(TestDerive)]
struct Test {
    #[Simple(i32 = 1, u16 = 2, float = 1.1, string = "test", enum2 = "aaa")]
    a: i32,
    b: i32,
}

#[test]
pub fn test_no_filed() {
    let attr: Simple = get_attribute!(Test::a, Simple);
    assert_eq!(attr.int32, 1);
    assert_eq!(attr.unsigned16, 2);
    assert_eq!(attr.float, 1.1);
    assert_eq!(attr.string, "test");
    assert_eq!(attr.enum1, Some(TestEnum::VariantC));
    assert_eq!(attr.enum2, TestEnum::VariantA);
    let a = Test {
        a: attr.int32,
        b: attr.int32,
    };
    println!("{}-{}", a.a, a.b);
}
