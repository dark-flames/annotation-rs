# Yui

Yui is an attribute reader generator for Rust.

## Features

### Attribute structure
Yui provides a derive macro `YuiAttribute` to create attribute structure by struct, `StructStruct`, `TupleStruct` and `NoFieldStruct` are all supported.
```rust
use yui::YuiAttribute;

#[derive(YuiAttribute)]
struct NoField;

#[derive(YuiAttribute)]
struct Tuple(i32, String);

#[derive(YuiAttribute)]
struct Struct {
    int: i32,
    float: f64,
    bool: bool,
}
```

#### Types
* String: `String` in Rust.
* Bool: `bool` in Rust.
* Integer: any integer types in Rust.
* Float: any float types in Rust.
* Object: other attribute structure.
* Enum: defined enum, remember to use `enum_value=true`option.
* Vec<T>: Vec of T(T can`t be Object, Vec or HashMap).
* HashMap<String, T>: HashMap of T mapping by `String` Skey.
If you want to make a field optional, use `Option<T>` on the field type.

```rust
use yui::{YuiEnumValue, YuiAttribute};

#[derive(YuiAttribute)]
struct Bar;

#[derive(YuiEnumValue)]
enum SomeEnum {
    A,
    B
}

#[derive(YuiAttribute)]
struct Foo {
    pub string: String,
    pub bool: bool,
    pub int: i32, // or other integer types like u32 ...
    pub float: f32, // or other float types like f64
    pub object: Bar, // any defined object
    #[attribute_field(enum_value=true)]
    pub enum_field: SomeEnum, // have to add enum_value option
    pub list: Vec<i32>, // nested type of vec can`t be Object, Vec or HashMap
    pub map: std::collections::HashMap<String, SomeEnum>,
    pub optional: Option<i32> // optional field
}
```
#### Options
* `path`\
    Generated reader will parse the field with the given name instead of its field name in Rust.
    ```rust
    #[derive(YuiAttribute)]
    struct Fool {
        #[attribute_field(path = "i32")]
        pub int32: i32,
    }
    ```
* `default`\
    Set the default value for this field. If the value is not present when parsing, the default value will be set to the field, even the field is optional.`Object`, `Vec` or `HashMap` fields can`t have default value.
    ```rust
    #[derive(YuiAttribute)]
    struct Fool {
        #[attribute_field(default = 1024)]
        pub int32: i32
    }
    ```
* `enum_value`\
    use `enum_value=true` on Enum type field.
        
#### Enum
Use derive `YuiEnumValue` on Enum to create a Enum value type.
```rust
use yui::YuiEnumValue;

#[derive(YuiEnumValue)]
enum SomeEnum {
    A,
    B
}
```
And then, the enum can be used as a field type.
* `variant_value` attribute\
    Customize a string corresponding value to variant(default is the snake case of variant name in Rust).
```rust
use yui::YuiEnumValue;

#[derive(YuiEnumValue)]
enum SomeEnum {
    #[variant_value("aaa")] // default is 'a'
    A,
    B
}
```   
### Parse attributes with `syn`and`quote`
`yui::AttributeStructs<T>` can be used in `parse_macro_input!`
```rust
let attributes = syn::parse_macro_inpit!(input as yui::AttributeStructs<Fool>);
```
If you want to parse attribute from `syn::Meta`, use `yui::AttributeStruct::from_meta()`.\
And attribute structure with value can be convert to token automatically. But the visibility of each field must be public.
```rust
quote::quote! {
    #attribute_filed
}
```

### Generate derive macro
If you want to use builtin reader generator, enable `generate-reader` feature.
Macro `generate_reader` is used to generate a derive macro.
```rust
use yui::generate_reader;

generated_reader!(
    MyDerive,
    [StructAttribute1, StructAttribute2],
    [FieldAttribute1, FieldAttribute2]
);

```
The macro will generate a public derive, it can be use to read attributes of `struct` ,`enum` or `union`, and record the metadata by generate `impl` block.

### Read attributes
Use the generated derive macro on a struct, and you can use the macro `has_attribute` and `get_attribute`to process attributes of the struct.
```rust
use yui::{get_attribute, has_attribute};

#[derive(MyDerive)]
#[StructAttribute1("some parameters")]
struct Foo {
    #[FieldAttribute1("some parameters")]
    field: i32
}

fn some_fn() {
    assert!(has_attribute(Foo, StructAttribute1));
    assert!(has_attribute(Foo::field, FieldAttribute1));
    let struct_attr1: Option<StructAttribute1> = get_attribute!(Fool, StructAttribute1);
    let field_attr1: Option<StructAttribute1> = get_attribute!(Fool::field, StructAttribute1);
}
```



