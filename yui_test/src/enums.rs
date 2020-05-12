#[macro_use]
use yui_derive::YuiEnumValue;

#[derive(YuiEnumValue, PartialEq, Debug)]
pub enum FetchType {
    Eager,
    LazyFetch,
}

#[derive(YuiEnumValue, PartialEq, Debug)]
#[attribute()]
pub enum ForeignKeyType {
    #[variant_value("one_to_one")]
    OneToOneForeignKey,
    #[variant_value("one_to_many")]
    OneToManyForeignKey,
    #[variant_value("many_to_one")]
    ManyToOneForeignKey,
    #[variant_value("many_to_many")]
    ManyToManyForeignKey,
}
