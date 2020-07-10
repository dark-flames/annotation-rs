use yui::YuiEnumValue;

#[cfg(test)]
use std::str::FromStr;

#[derive(YuiEnumValue, Debug, PartialEq, Clone)]
#[mod_path = "yui_test_attribute::enums"]
pub enum TestEnum {
    #[variant_value("aaa")]
    VariantA,
    VariantB,
    VariantC,
}

#[test]
pub fn test_enum() {
    assert_eq!(TestEnum::from_str("aaa").unwrap(), TestEnum::VariantA);
    assert_eq!(TestEnum::from_str("variant_b").unwrap(), TestEnum::VariantB);
    assert_eq!(TestEnum::from_str("variant_c").unwrap(), TestEnum::VariantC);
}
