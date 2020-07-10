use crate::enums::TestEnum;
use std::collections::HashMap;

use yui::YuiAttribute;

#[derive(YuiAttribute, Clone)]
#[mod_path = "yui_test_attribute::attribute"]
pub struct NoField;

#[derive(YuiAttribute, Clone)]
#[mod_path = "yui_test_attribute::attribute"]
pub struct Simple {
    #[attribute_field(alias = "i32")]
    pub int32: i32,
    #[attribute_field(alias = "u16")]
    pub unsigned16: u16,
    pub float: f32,
    pub string: String,
    #[attribute_field(enum_value = true, default = "variant_c")]
    pub enum1: Option<TestEnum>,
    #[attribute_field(enum_value = true)]
    pub enum2: TestEnum,
}

#[derive(YuiAttribute, Clone)]
#[mod_path = "yui_test_attribute::attribute"]
pub struct Tuple(pub Option<String>);

#[derive(YuiAttribute, Clone)]
#[mod_path = "yui_test_attribute::attribute"]
pub struct Full {
    pub object: Simple,
    pub vector: Vec<String>,
    #[attribute_field(enum_value = true)]
    pub map: HashMap<String, TestEnum>,
    pub map2: HashMap<String, Tuple>,
    #[attribute_field(enum_value = true)]
    pub map3: HashMap<String, Vec<TestEnum>>,
}
