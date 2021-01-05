# annotation-rs

Compile-time annotation parser for rust

## Features

### Annotation
annotation-rs provides a derive macro `Annotation` to create annotation structure by struct, `StructStruct`, `TupleStruct` and `NoFieldStruct` are all supported.
```rust
use annotation_rs::Annotation;

#[derive(Annotation)]
struct NoField;

#[derive(Annotation)]
struct Tuple(i32, String);

#[derive(Annotation)]
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
* Object: other annotation structure.
* Enum: defined enum, remember to use `enum_value=true`option.
* Vec<T>: Vec of T(T can`t be Object, Vec or HashMap).
* HashMap<String, T>: HashMap of T mapping by `String` Skey.
If you want to make a field optional, use `Option<T>` on the field type.

```rust
use annotation_rs::{AnnotationEnumValue, Annotation};

#[derive(Annotation)]
struct Bar;

#[derive(AnnotationEnumValue)]
enum SomeEnum {
    A,
    B
}

#[derive(Annotation)]
struct Foo {
    pub string: String,
    pub bool: bool,
    pub int: i32, // or other integer types like u32 ...
    pub float: f32, // or other float types like f64
    pub object: Bar, // any defined object
    #[field(enum_value=true)]
    pub enum_field: SomeEnum, // have to add enum_value option
    pub list: Vec<i32>, // nested type of vec can`t be Object, Vec or HashMap
    pub map: std::collections::HashMap<String, SomeEnum>,
    pub optional: Option<i32> // optional field
}
```
#### Options
* `alias`\
    Generated reader will parse the field with the given name instead of its field name in Rust.
    ```rust
    #[derive(Annotation)]
    struct Foo {
        #[field(alias = "i32")]
        pub int32: i32,
    }
    ```
* `default`\
    Set the default value for this field. If the value is not present when parsing, the default value will be set to the field, even the field is optional.`Object`, `Vec` or `HashMap` fields can`t have default value.
    ```rust
    #[derive(Annotation)]
    struct Foo {
        #[field(default = 1024)]
        pub int32: i32
    }
    ```
* `enum_value`\
    use `enum_value=true` on Enum type field.
        
#### Enum
Use derive `AnnotationEnumValue` on Enum to create a Enum value type.
```rust
use annotation_rs::AnnotationEnumValue;

#[derive(AnnotationEnumValue)]
enum SomeEnum {
    A,
    B
}
```
And then, the enum can be used as a field type.
* `variant_value` attribute\
    Customize a string corresponding value to variant(default is the snake case of variant name in Rust).
```rust
use annotation_rs::AnnotationEnumValue;

#[derive(AnnotationEnumValue)]
enum SomeEnum {
    #[variant_value("aaa")] // default is 'a'
    A,
    B
}
```   
### Parse annotations with `syn`and`quote`
`annotation_rs::AnnotationStructures<T>` can be used in `parse_macro_input!`
```rust
let annotations = syn::parse_macro_inpit!(input as annotation_rs::AnnotationStructures<Foo>);
```
If you want to parse annotation from `syn::Meta`, use `annotation_rs::AnnotationStructure::from_meta()`.\
And annotation structure with value can be convert to token automatically. But the visibility of each field must be public.
```rust
use proc_macro::TokenStream;

#[derive(Annotation)]
struct Foo {
    #[field(default = 1024)]
    pub int32: i32
}

fn derive_fn(input: TokenStream) -> TokenStream {
    let annotations = syn::parse_macro_input!(input as annotation_rs::AnnotationStructures<Foo>);
    let attrs = annotations.attrs;

    TokenStream::from(quote::quote! {
        fn get_attrs() -> Vec<Foo> {
            vec![#(#attrs),*]
        }
    })
}
```

### Generate derive macro
If you want to use builtin reader generator, enable `annotation_reader` feature.
Macro `generate_reader` is used to generate a derive macro.
```rust
use annotation_rs::generate_reader;

generated_reader!(
    MyDerive,
    [StructAttribute1, StructAttribute2],
    [FieldAttribute1, FieldAttribute2]
);

```
The macro will generate a public derive, it can be use to read annotations of `struct` ,`enum` or `union`, and record the metadata by generate `impl` block.

### Read annotations
Use the generated derive macro on a struct, and you can use the macro `has_annotation` and `get_annotation`to process annotations of the struct.
The feature require nightly rustc because `proc_macro_hygiene` is required.
```rust
#![feature(proc_macro_hygiene)]
use annotation_rs::{get_annotation, has_annotation};

#[derive(MyDerive)]
#[StructAttribute1("some parameters")]
struct Foo {
    #[FieldAttribute1("some parameters")]
    field: i32
}

fn some_fn() {
    assert!(has_annotation!(Foo, StructAttribute1));
    assert!(has_annotation!(Foo::field, FieldAttribute1));
    let struct_attr1: Option<StructAttribute1> = get_annotation!(Foo, StructAttribute1);
    let field_attr1: Option<StructAttribute1> = get_annotation!(Foo::field, StructAttribute1);
}
```



