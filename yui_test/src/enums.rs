use yui::YuiEnumValue;

#[derive(YuiEnumValue, Debug, PartialEq)]
pub enum TestEnum {
    #[variant_value("aaa")]
    VariantA,
    VariantB,
    VariantC,
}
