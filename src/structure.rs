use crate::error::Error;
use crate::helper::get_lit_str;
use crate::symbol::Symbol;
use heck::SnakeCase;
use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Meta, Variant};

pub struct EnumItem {
    ident: String,
    value: String,
}

impl EnumItem {
    pub fn from_ast(input: &Variant) -> Result<Self, Error> {
        let ident = input.ident.to_string().clone();
        let mut value = ident.to_snake_case();

        if !input.attrs.is_empty() {
            const VALUE_PATH: Symbol = Symbol("value");

            for attr in input.attrs.iter() {
                match attr.parse_meta() {
                    Meta::NameValue(name_value) => {
                        if name_value.path == VALUE_PATH {
                            value = get_lit_str(&name_value.lit, &name_value.path)?
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            "EnumItem can only be create from MetaNameValueStr",
                        ))
                    }
                };
            }
        }

        Ok(EnumItem { ident, value })
    }

    pub fn to_pattern_token_stream(&self, enum_name: &String) -> TokenStream {
        let item_value = self.value.as_str();
        let item_ident = self.ident.as_str();
        quote! {
            #item_value => Ok(enum_name::item_ident)
        }
    }
}

pub struct EnumValue {
    name: String,
    items: Vec<EnumItem>,
}

impl EnumValue {
    pub fn from_ast(input: &DeriveInput) -> Result<Self, Error> {
        match &input.data {
            Data::Enum(enum_ast) => {
                let items: Result<Vec<EnumItem>, Error> = enum_ast
                    .variants
                    .iter()
                    .map(|item_ast| EnumItem::from_ast(item_ast))
                    .collect();

                Ok(EnumValue {
                    name: input.ident.to_string().clone(),
                    items: items?,
                })
            }
            _ => Err(Error::new("EnumValue can only be created from Enum")),
        }
    }

    pub fn get_lit_reader(&self) -> TokenStream {
        let enum_name = self.name.as_str();
        let arms: Vec<TokenStream> = self
            .items
            .iter()
            .map(|item| item.to_pattern_token_stream(&self.name))
            .collect();
        quote! {
            impl yukino_attribute_reader::traits::ValueEnum #enum_name {
                pub fn read_from_lit(lit: &syn::Lit, path: &syn::Path) -> Self {
                    match yukino_attribute_reader::helper::get_lit_str(lit, path)?.as_str() {
                        #(#arm),*
                        others => Err(
                            yukino_attribute_reader::Error::from_syn_error(
                                syn::Error::new_spanned(
                                    lit,
                                    format!("Unexpected {} value: {}", #enum_name, path),
                                )
                            )
                        )
                    }
                }
            }
        }
    }
}

pub enum Type {
    String,
    Bool,
    Number,
    Object(String),
    Enum(EnumValue),
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
