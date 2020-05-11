#[macro_use]
use yui_derive::EnumValue;
use std::str::FromStr;

#[derive(EnumValue, PartialEq, Debug)]
pub enum TestEnum {
    #[enum_item_value("aaa")]
    ItemA,
    ItemB,
    ItemC,
}

#[test]
pub fn test_impl_of_enum() {
    assert_eq!(TestEnum::from_str("aaa").unwrap(), TestEnum::ItemA);
    assert_eq!(TestEnum::from_str("item_b").unwrap(), TestEnum::ItemB);
    assert_eq!(TestEnum::from_str("item_c").unwrap(), TestEnum::ItemC)
}
