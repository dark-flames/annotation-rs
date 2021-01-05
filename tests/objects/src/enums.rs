use derive::AnnotationEnumValue;

#[cfg(test)]
use std::str::FromStr;

#[derive(AnnotationEnumValue, Debug, PartialEq, Clone)]
#[mod_path = "objects::enums"]
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
