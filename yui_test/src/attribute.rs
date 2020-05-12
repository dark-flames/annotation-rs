#[macro_use]
use yui_derive::YuiAttribute;
use crate::enums::ForeignKeyType;
use std::collections::HashMap;

#[derive(YuiAttribute)]
struct PrimaryKey;

#[derive(YuiAttribute)]
struct Person {
    pub name: String,
    pub age: u8,
    pub alive: bool,
    pub weight: f32,
}

#[derive(YuiAttribute)]
struct Position(i32, i32);

#[derive(YuiAttribute)]
#[attribute("JoinColumn")]
pub struct JoinColumnAttributeStructure {
    pub name: Option<String>,
    pub referenced_column: Option<String>,
    #[attribute_field(defualt = false)]
    pub unique: bool,
    #[attribute_field(enum_value = true)]
    pub foreign_key: ForeignKeyType,
    #[attribute_field(path = "options")]
    pub options_map: HashMap<String, String>,
}

#[derive(YuiAttribute)]
#[attribute("JoinTable")]
pub struct JoinTableAttributeStructure {
    pub name: Option<String>,
    pub referenced_table: String,
    pub columns: HashMap<String, JoinColumnAttributeStructure>,
}
