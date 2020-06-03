#![feature(proc_macro_hygiene)]
use yui::get_attribute;
use yui_test::FullDerive;
use yui_test_attribute::attribute::Simple;
use yui_test_attribute::attribute::Full;
use yui_test_attribute::enums::TestEnum;


#[derive(FullDerive)]
#[Full(
    object(i32 = 1, u16 = 2, float = 1.1, string = "test", enum2 = "aaa"),
    vector("1", "2", "3"),
    map(a="aaa", b="variant_b", c="variant_c")
)]
struct Test;

#[test]
pub fn test_full() {
    let attr: Full = get_attribute!(Test, Full).unwrap();
    let simple = attr.object;
    assert_eq!(simple.int32, 1);
    assert_eq!(simple.unsigned16, 2);
    assert_eq!(simple.float, 1.1);
    assert_eq!(simple.string, "test");
    assert_eq!(simple.enum1, Some(TestEnum::VariantC));
    assert_eq!(simple.enum2, TestEnum::VariantA);
    assert_eq!(attr.vector, ["1", "2", "3"]);
    assert_eq!(attr.map["a"], TestEnum::VariantA);
    assert_eq!(attr.map["b"], TestEnum::VariantB);
    assert_eq!(attr.map["c"], TestEnum::VariantC);
}