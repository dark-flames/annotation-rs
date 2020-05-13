use std::str::FromStr;
use yui_test::enums::TestEnum;

#[test]
pub fn test_enum() {
    assert_eq!(TestEnum::from_str("aaa").unwrap(), TestEnum::VariantA);
    assert_eq!(TestEnum::from_str("variant_b").unwrap(), TestEnum::VariantB);
    assert_eq!(TestEnum::from_str("variant_c").unwrap(), TestEnum::VariantC);
}
