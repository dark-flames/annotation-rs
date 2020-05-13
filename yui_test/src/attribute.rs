use crate::enums::TestEnum;
use std::collections::HashMap;
use yui_derive::YuiAttribute;

pub struct TestNoFieldStruct;

#[derive(YuiAttribute)]
pub struct TestSimpleStruct {
    #[attribute_field(path = "test_i32")]
    pub test_int32: i32,
    #[attribute_field(path = "test_u16")]
    pub test_unsigned16: u16,
    pub test_float: f32,
    pub test_string: String,
    #[attribute_field(enum_value = true, default = "variant_c")]
    pub test_enum1: Option<TestEnum>,
    #[attribute_field(enum_value = true)]
    pub test_enum2: TestEnum,
}

#[derive(YuiAttribute)]
#[attribute("test_tuple")]
pub struct TestTuple(i32, Option<String>);

#[derive(YuiAttribute)]
pub struct TestStruct {
    pub object: TestSimpleStruct,
    pub vector: Vec<String>,
    #[attribute_field(enum_value = true)]
    pub map: HashMap<String, TestEnum>,
}
