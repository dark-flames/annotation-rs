use syn::export::fmt::Error;

pub struct EnumItem {
    ident: String,
    value: String,
}

pub struct Enum {
    name: String,
    items: Vec<EnumItem>,
}

pub enum Type {
    String,
    Bool,
    Number,
    Object(String),
    Enum(Enum),
    List(Type),
}

pub enum FieldType {
    OptionalField(Type),
    RequiredField(Type),
}

impl FieldType {
    pub fn is_required(&self) -> bool {
        match self {
            FieldType::OptionalField(_) => false,
            FieldType::RequiredField(_) => true,
        }
    }

    pub fn unwrap(&self) -> &Type {
        match self {
            FieldType::OptionalField(field_type) => field_type,
            FieldType::RequiredField(field_type) => field_type,
        }
    }
}

pub struct Field {
    name: String,
    field_type: FieldType,
}

pub struct Attribute {
    name: String,
    fields: Vec<Field>,
}
