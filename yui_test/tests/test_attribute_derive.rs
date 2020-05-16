use yui_test::NoField;

#[derive(NoField)]
#[TestNoFieldStruct]
struct Test {
    a: i32,
}

#[test]
pub fn test() {
    assert_eq!(Test::count(), 1)
}
