use std::str::FromStr;
use yui_test::enums::{FetchType, ForeignKeyType};

#[test]
pub fn test_basic_map() {
    assert_eq!(FetchType::from_str("eager").unwrap(), FetchType::Eager);
    assert_eq!(
        FetchType::from_str("lazy_fetch").unwrap(),
        FetchType::LazyFetch
    );
}

#[test]
pub fn test_rename_variant_value() {
    assert_eq!(
        ForeignKeyType::from_str("one_to_one").unwrap(),
        ForeignKeyType::OneToOneForeignKey
    );
    assert_eq!(
        ForeignKeyType::from_str("one_to_many").unwrap(),
        ForeignKeyType::OneToManyForeignKey
    );
    assert_eq!(
        ForeignKeyType::from_str("many_to_one").unwrap(),
        ForeignKeyType::ManyToOneForeignKey
    );
    assert_eq!(
        ForeignKeyType::from_str("many_to_many").unwrap(),
        ForeignKeyType::ManyToManyForeignKey
    );
}
