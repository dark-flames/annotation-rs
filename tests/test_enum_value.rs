#[macro_use]
use yui;

#[derive(EnumValue)]
enum TestEnum {
    #[enum_item_value("aaa")]
    A,
    B,
    C,
}
#[test]
pub fn test_impl_of_enum() {
    let value: TestEnum = TestEnum::from_str("aaa")?;

    assert_eq!(value, TestEnum::A);
}
