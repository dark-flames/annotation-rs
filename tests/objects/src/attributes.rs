use crate::enums::TestEnum;
use std::collections::HashMap;

use derive::Annotation;

#[derive(Annotation, Clone)]
#[mod_path = "objects::attributes"]
pub struct NoField;

#[derive(Annotation, Clone)]
#[mod_path = "objects::attributes"]
pub struct Simple {
    #[field(alias = "i32")]
    pub int32: i32,
    #[field(alias = "u16")]
    pub unsigned16: u16,
    pub float: f32,
    pub string: String,
    #[field(enum_value = true, default = "variant_c")]
    pub enum1: Option<TestEnum>,
    #[field(enum_value = true)]
    pub enum2: TestEnum,
}

#[derive(Annotation, Clone)]
#[mod_path = "objects::attributes"]
pub struct Tuple(pub Option<String>);

#[derive(Annotation, Clone)]
#[mod_path = "objects::attributes"]
pub struct Full {
    pub object: Simple,
    pub vector: Vec<String>,
    #[field(enum_value = true)]
    pub map: HashMap<String, TestEnum>,
    pub map2: HashMap<String, Tuple>,
    #[field(enum_value = true)]
    pub map3: HashMap<String, Vec<TestEnum>>,
}
