use proc_macro2::TokenStream;
use quote::quote;
use std::fmt;
use syn::{Error, Field, Ident, PathSegment, Type as SynType, TypePath};

use super::helper::{
    get_nested_type, unwrap_punctuated_first, unwrap_punctuated_last, unwrap_type_path,
};
use crate::helper::get_nested_types;

pub enum Type {
    String,
    Bool,
    Integer(Ident),
    Float(Ident),
    Object(Ident),
    Enum(Ident),
    List(Box<Type>),
    Map(Box<Type>),
}

impl Type {
    fn get_nested_type_path(segment: &'a PathSegment) -> Result<Vec<&'a TypePath>, Error> {
        get_nested_types(&segment, "Unexpect Arguments")?
            .iter()
            .map(|&ty| unwrap_type_path(ty, "Argument of HashMap or Vec must be type path"))
            .collect()
    }

    pub fn from_ast(type_path: &TypePath, is_enum: bool) -> Result<Self, Error> {
        let segment = unwrap_punctuated_first(
            &type_path.path.segments,
            Error::new_spanned(type_path, "Unexpected type path segment"),
        )?;

        let token = segment.ident.clone();

        match token.to_string().as_str() {
            "String" => Ok(Type::String),
            "bool" => Ok(Type::Bool),
            "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => {
                Ok(Type::Integer(token))
            }
            "f32" | "f64" => Ok(Type::Float(token)),
            "Vec" => {
                let nested_type_paths = Self::get_nested_type_path(&segment)?;

                match nested_type_paths.first() {
                    Some(&nested_type_path) => Ok(Type::List(Box::new(Self::from_ast(
                        nested_type_path,
                        is_enum,
                    )?))),
                    None => Err(Error::new_spanned(
                        segment,
                        "Vec need at least one argument",
                    )),
                }
            }
            "HashMap" => {
                let nested_type_paths = Self::get_nested_type_path(&segment)?;

                match nested_type_paths.first() {
                    Some(&key_type_path) => {
                        let key_segment = unwrap_punctuated_first(
                            &key_type_path.path.segments,
                            Error::new_spanned(key_type_path, "HashMap need at least two key"),
                        )?;
                        if key_segment.ident.to_string().as_str() != "String" {
                            return Err(Error::new_spanned(
                                segment,
                                "Key of HashMap must be String type",
                            ));
                        }
                    }
                    None => return Err(Error::new_spanned(segment, "HashMap need two argument")),
                }

                match nested_type_paths.last() {
                    Some(&nested_type_path) => Ok(Type::Map(Box::new(Self::from_ast(
                        nested_type_path,
                        is_enum,
                    )?))),
                    None => Err(Error::new_spanned(
                        segment,
                        "HashMap need at least two argument",
                    )),
                }
            }
            type_name => Ok(match is_enum {
                true => Type::Enum(token),
                false => Type::Object(token),
            }),
        }
    }

    pub fn get_token_stream(&self) -> TokenStream {
        match self {
            Type::String => quote! { String },
            Type::Bool => quote! { bool },
            Type::Integer(ident) => quote! { #ident },
            Type::Float(ident) => quote! { #ident },
            Type::Object(ident) => quote! { #ident },
            Type::Enum(ident) => quote! { #ident },
            Type::List(ident) => {
                let nested_token_stream = ident.get_token_stream();
                quote! { Vec<#nested_token_stream> }
            }
            Type::Map(ident) => {
                let nested_token_stream = ident.get_token_stream();
                quote! { std::collections::HashMap<String, #nested_token_stream> }
            }
        }
    }

    pub fn get_nested_pattern(&self, named: bool) -> TokenStream {
        match (self, named) {
            (Type::String, true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            },
            (Type::String, false) => quote! {
                syn::NestedMeta::Lit(lit)
            },
            (Type::Bool, true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            },
            (Type::Bool, false) => quote! {
                syn::NestedMeta::Lit(lit)
            },
            (Type::Integer(_), true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            },
            (Type::Integer(_), false) => quote! {
                syn::NestedMeta::Lit(lit)
            },
            (Type::Float(_), true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            },
            (Type::Float(_), false) => quote! {
                syn::NestedMeta::Lit(lit)
            },
            (Type::Object(_), true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(meta_value)
                )
            },
            (Type::Enum(_), true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(meta_value)
                )
            },
            (Type::Enum(_), false) => quote! {
                syn::NestedMeta::Lit(lit)
            },
            (Type::List(_), true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(meta_value)
                )
            },
            (Type::Map(_), true) => quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(meta_value)
                )
            },
            _ => unreachable!(),
        }
    }

    pub fn get_lit_reader_token_stream(
        &self,
        lit: TokenStream,
        path: TokenStream,
        item: TokenStream,
    ) -> TokenStream {
        match self {
            Type::String => quote! {
                yui::get_lit_str(&#lit, &#path)
            },
            Type::Bool => quote! {
                yui::get_lit_bool(&#lit, &#path)
            },
            Type::Integer(_) => {
                let result_type = self.get_token_stream();
                quote! {
                    yui::get_lit_int::<#result_type>(&#lit, &#path)
                }
            }
            Type::Float(_) => {
                let result_type = self.get_token_stream();
                quote! {
                    yui::get_lit_float::<#result_type>(&#lit, &#path)
                }
            }
            Type::Object(_) => {
                let result_type = self.get_token_stream();
                quote! {
                    #result_type::from_meta_list(&#item)
                }
            }
            Type::Enum(_) => {
                quote! {
                    yui::get_lit_str(&#lit, &#path)?.parse().map_err(
                        |e: yui::Error| {
                            syn::Error::new_spanned(&#lit, e.get_message())
                        }
                    )
                }
            }
            Type::List(ty) => {
                let result_type = ty.get_token_stream();
                let pattern = ty.get_nested_pattern(false);
                let reader =
                    ty.get_lit_reader_token_stream(quote! {lit}, quote! {meta_value.path}, item);
                quote! {
                    meta_value.nested.iter().map(|meta_nested_meta| {
                        match &meta_nested_meta {
                            #pattern => #reader,
                            _ => Err(syn::Error::new_spanned(
                                meta_nested_meta,
                                "Only support List of Lit"
                            ))
                        }
                    }).collect::<Result<Vec<#result_type>, syn::Error>>()
                }
            }
            Type::Map(ty) => {
                let result_type = ty.get_token_stream();
                let pattern = ty.get_nested_pattern(true);
                let reader = ty.get_lit_reader_token_stream(
                    quote! {meta_value.lit},
                    quote! {meta_value.path},
                    quote! {meta_value},
                );
                quote! {
                    {
                        let value_pairs: Result<Vec<(String, #result_type)>, syn::Error> = meta_value.nested.iter().map(|meta_nested_meta| {
                            match &meta_nested_meta {
                                #pattern => {
                                    Ok((
                                         format!("{}",yui::unwrap_punctuated_first(
                                                &meta_value.path.segments,
                                                syn::Error::new_spanned(
                                                    meta_value,
                                                    "Unexpected type path segment"
                                                )
                                             )?.ident
                                         ),
                                         #reader?
                                    ))
                                },
                                _ => Err(syn::Error::new_spanned(
                                    meta_nested_meta,
                                    "Only support List of Lit"
                                ))
                            }
                        }).collect();

                        value_pairs.map(|pairs| {
                            pairs.into_iter().collect::<std::collections::HashMap<String, #result_type>>()
                        })
                    }
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
            Type::Integer(ty) => write!(f, "{}", ty),
            Type::Float(ty) => write!(f, "{}", ty),
            Type::Object(ty) => write!(f, "{}", ty),
            Type::Enum(ty) => write!(f, "{}", ty),
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
            Error::new_spanned(type_path, "Unexpected type path segment"),
        )?;

        match segment.ident.to_string().as_str() {
            "Option" => {
                let nested_type = get_nested_type(segment, "Unexpected type path Argument")?;

                let nested_type_path =
                    unwrap_type_path(nested_type, "Type in Option must be TypePath")?;

                Ok(FieldType::OptionalField(Type::from_ast(
                    nested_type_path,
                    is_enum,
                )?))
            }
            _ => Ok(FieldType::RequiredField(Type::from_ast(
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
    pub fn from_string(value: String, lit: &Field, ty: &Type) -> Result<Self, Error> {
        match ty {
            Type::String => Ok(DefaultValue::String(value)),
            Type::Bool => Ok(DefaultValue::Bool(value)),
            Type::Integer(_) => Ok(DefaultValue::Integer(value)),
            Type::Float(_) => Ok(DefaultValue::Float(value)),
            Type::Enum(_) => Ok(DefaultValue::Enum(value)),
            _ => Err(Error::new_spanned(
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
