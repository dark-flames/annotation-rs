use proc_macro2::TokenStream;
use quote;
use std::fmt;
use syn::{Path, Type as SynType, TypePath};

use super::helper::{
    get_lit_bool_str, get_lit_float_str, get_lit_int_str, get_nested_type, unwrap_punctuated_first,
    unwrap_punctuated_last, unwrap_type_path,
};
use super::Error;
use super::{get_lit_float, get_lit_int, get_lit_str};

pub enum Type {
    String,
    Bool,
    Integer(String),
    Float(String),
    Object(String),
    Enum(String),
    List(Type),
    Map(Type),
}

impl Type {
    fn get_list_nested_type(type_path: &TypePath, is_enum: bool) -> Result<Type, Error> {
        let path_segment = unwrap_punctuated_first(
            &type_path.path.segments,
            Error::from_syn_error(&type_path, "Unexpected type path segment"),
        )?;

        let nested_type = get_nested_type(path_segment, "Unexpected type path Argument")?;

        match nested_type {
            SynType::Path(type_path) => {
                let mut ty = Type::from_ast(type_path, is_enum);
                if ty.is_ok() {
                    if !match ty.unwrap() {
                        Type::String
                        | Type::Bool
                        | Type::Integer(_)
                        | Type::Float(_)
                        | Type::Enum(_) => true,
                        _ => false,
                    } {
                        ty = Err(Error::from_syn_error(
                            type_path,
                            "Nested type in Vec can not be Object, Vec or Map, because these can not create from lit"
                        ))
                    }
                }
                ty
            }
            _ => Error::from_syn_error(type_path, "Nested type must be type path"),
        }
    }

    fn get_map_nested_type(type_path: &TypePath, is_enum: bool) -> Result<Type, Error> {
        let path_last_segment = unwrap_punctuated_last(
            &type_path.path.segments,
            Error::from_syn_error(&type_path, "Unexpected type path segment"),
        )?;

        let path_first_segment = unwrap_punctuated_first(
            &type_path.path.segments,
            Error::from_syn_error(&type_path, "Unexpected type path segment"),
        )?;

        let nested_type = get_nested_type(path_last_segment, "Unexpected type path Argument")?;

        let first_nested_type =
            get_nested_type(path_first_segment, "Unexpected type path Argument")?;

        let first_type_ident = &match first_nested_type {
            SynType::Reference(reference) => match *reference.elem {
                SynType::Path(first_type_path) => unwrap_punctuated_last(
                    &first_type_path.path.segments,
                    Error::from_syn_error(&type_path, "Unexpected type path segment"),
                ),
                _ => Err(Error::from_syn_error(
                    type_path,
                    "Nested type must be type path",
                )),
            },
            _ => Err(Error::from_syn_error(
                type_path,
                "First nested type of HasMap must be &str",
            )),
        }?
        .ident;

        if first_type_ident != String::from("str") {
            return Err(Error::from_syn_error(
                type_path,
                "First nested type of HasMap must be &str",
            ));
        }

        match nested_type {
            SynType::Path(type_path) => Type::from_ast(type_path, is_enum),
            _ => Err(Error::from_syn_error(
                type_path,
                "Nested type must be type path",
            )),
        }
    }

    pub fn from_ast(type_path: &TypePath, is_enum: bool) -> Result<Self, Error> {
        let segment = unwrap_punctuated_first(
            &type_path.path.segments,
            Error::from_syn_error(type_path, "Unexpected type path segment"),
        )?;

        let token = segment.ident.to_string().as_str();

        match token {
            "String" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => {
                Ok(Type::Integer(String::from(token)))
            }
            "f32" | "f64" => Ok(Type::Float(String::from(token))),
            "Vec" => Ok(Type::List(Self::get_list_nested_type(type_path, is_enum)?)),
            "HashMap" => Ok(Type::Map(Self::get_map_nested_type(type_path, is_enum)?)),
            type_name if type_name.chars().next().unwrap().is_uppercase() => Ok(match is_enum {
                true => Type::Enum(String::from(type_name)),
                false => Type::Object(String::from(type_name)),
            }),
            _ => Err(Error::from_syn_error(type_path, "Unexpected type token")),
        }
    }

    pub fn get_nested_pattern(&self, named: bool) -> Result<TokenStream, Error> {
        match (self, named) {
            (Type::String, true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            }),
            (Type::String, false) => OK(quote! {
                syn::NestedMeta::Lit(lit)
            }),
            (Type::Bool, true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            }),
            (Type::Bool, false) => OK(quote! {
                syn::NestedMeta::Lit(lit)
            }),
            (Type::Integer(_), true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            }),
            (Type::Integer(_), false) => OK(quote! {
                syn::NestedMeta::Lit(lit)
            }),
            (Type::Float(_), true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            }),
            (Type::Float(_), false) => OK(quote! {
                syn::NestedMeta::Lit(lit)
            }),
            (Type::Object(_), true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(meta_value)
                )
            }),
            (Type::Enum(_), true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            }),
            (Type::Enum(_), false) => OK(quote! {
                syn::NestedMeta::Lit(lit)
            }),
            (Type::List(_), true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(meta_value)
                )
            }),
            (Type::Map(_), true) => OK(quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(meta_value)
                )
            }),
            _ => Err(Error::new("Invalid field type and named")),
        }
    }

    pub fn get_lit_reader_token_stream(&self, lit: &str, path: &str, item: &str) -> TokenStream {
        match self {
            Type::String => quote! {
                yukino_attribute_reader::get_lit_str(#lit, #path)
            },
            Type::Bool => quote! {
                yukino_attribute_reader::get_lit_bool(#lit, #path)
            },
            Type::Integer(ty) => {
                let result_type = ty.as_str();
                quote! {
                    yukino_attribute_reader::get_lit_int::<#result_type>(#lit, #path)
                }
            }
            Type::Float(ty) => {
                let result_type = ty.as_str();
                quote! {
                    yukino_attribute_reader::get_lit_float::<#result_type>(#lit, #path)
                }
            }
            Type::Object(ty) => {
                let result_type = ty.as_str();
                quote! {
                    #result_type::from_meta_list(#item)
                }
            }
            Type::Enum(ty) => {
                let result_type = ty.as_str();
                quote! {
                    #result_type::read_from_lit(#lit, #path)
                }
            }
            Type::List(ty) => {
                let pattern = ty.get_nested_pattern(false);
                let reader = ty.get_lit_reader_token_stream("&lit", "&meta_value.path", item);
                quote! {
                    meta_value.nested.iter().map(|meta_nested_meta| {
                        match &meta_nested_meta {
                            #parttern => #reader,
                            _ => Err(yukino_attribute_reader::Error::from_syn_error(
                                meta_nested_meta,
                                "Only support List of Lit"
                            ))
                        }
                    })
                }
            }
            Type::Map(ty) => {
                let result_type = ty.to_string().as_str();
                let pattern = ty.get_nested_pattern(true);
                let reader = ty.get_lit_reader_token_stream(
                    "&meta_value.lit",
                    "&meta_value.path",
                    "&meta_value",
                );
                quote! {
                    Ok(meta_value.nested.iter().map(|meta_nested_meta| {
                        match &meta_nested_meta {
                            #parttern => {
                                OK((format!("{}", &meta_value.path.ident).as_str, #reader?))
                            },
                            _ => Err(yukino_attribute_reader::Error::from_syn_error(
                                meta_nested_meta,
                                "Only support List of Lit"
                            ))
                        }
                    }).collcet::<Result<Vec<(&str, #result_type)>, Error>>()?
                        .into_iter().collect::<HasMap<&str, #result_type>>())
                }
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::String => write!(f, "String"),
            Type::Bool => write!(f, "bool"),
            Type::Integer(ty) => write!(f, ty),
            Type::Float(ty) => write!(f, ty),
            Type::Object(ty) => write!(f, ty),
            Type::Enum(ty) => write!(f, ty),
            Type::List(ty) => write!(f, "Vec<{}>", ty),
            Type::Map(ty) => write!(f, "Map<&str, {}>", ty),
        }
    }
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

    pub fn from_ast(field_type: &SynType, is_enum: bool) -> Result<FieldType, Error> {
        let type_path = unwrap_type_path(field_type, "Field Type must be TypePath")?;

        let segment = unwrap_punctuated_first(
            &type_path.path.segments,
            Error::from_syn_error(type_path, "Unexpected type path segment"),
        )?;

        match segment.ident.to_string().as_str() {
            "Option" => {
                let nested_type = get_nested_type(segment, "Unexpected type path Argument")?;

                let nested_type_path =
                    unwrap_type_path(nested_type, "Type in Option must be TypePath")?;

                OK(FieldType::OptionalField(Type::from_ast(
                    nested_type_path,
                    is_enum,
                )?))
            }
            _ => OK(FieldType::RequiredField(Type::from_ast(
                type_path, is_enum,
            )?)),
        }
    }
}

pub enum DefaultValue {
    String(String),
    Bool(String),
    Integer(String),
    Float(String),
    Enum(String),
}

impl DefaultValue {
    pub fn from_lit(lit: &Lit, path: &Path, ty: &Type) -> Result<Self, Error> {
        match field_type {
            Type::String => Ok(DefaultValue::String(get_lit_str(lit, path)?)),
            Type::Bool => Ok(DefaultValue::Bool(get_lit_bool_str(lit, path)?)),
            Type::Integer(_) => Ok(DefaultValue::Integer(get_lit_int_str(lit, path)?)),
            Type::Float(_) => Ok(DefaultValue::Float(get_lit_float_str(lit, path)?)),
            Type::Enum(_) => Ok(DefaultValue::Enum(get_lit_str(lit, path)?)),
            _ => Err(Error::from_syn_error(
                lit,
                "Only support default value on String / Bool / Integer / Float / Enum",
            )),
        }
    }
}

impl fmt::Display for DefaultValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DefaultValue::String(value) => value,
                DefaultValue::Bool(value) => value,
                DefaultValue::Integer(value) => value,
                DefaultValue::Float(value) => value,
                DefaultValue::Enum(value) => value,
            }
        )
    }
}
