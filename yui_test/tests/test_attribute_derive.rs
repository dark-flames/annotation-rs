use yui_test::NoField;

#[derive(NoField)]
#[TestNoFieldStruct]
struct Test;

#[test]
pub fn test() {
    assert_eq!(Test::count(), 1)
}
