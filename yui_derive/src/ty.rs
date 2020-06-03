use proc_macro2::TokenStream;
use quote::{format_ident};
use std::fmt;
use syn::{Error, Field, Ident, PathSegment, Type as SynType, TypePath};
use super::reader::InterpolatedList;

use yui_internal::{get_nested_type, get_nested_types, unwrap_punctuated_first, unwrap_type_path};
use crate::reader::Interpolated;

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
    fn get_nested_type_path(segment: &PathSegment) -> Result<Vec<&TypePath>, Error> {
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
                    Some(&nested_type_path) => {
                        let nested_type = Box::new(Self::from_ast(nested_type_path, is_enum)?);
                        match *nested_type {
                            Type::Object(_) | Type::Map(_) | Type::List(_) => {
                                Err(Error::new_spanned(
                                    segment,
                                    "Vec can not nest Object, Map or List",
                                ))
                            }
                            _ => Ok(Type::List(nested_type)),
                        }
                    }
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
            _ => Ok(match is_enum {
                true => Type::Enum(token),
                false => Type::Object(token),
            }),
        }
    }

    pub fn get_type_token_stream(&self) -> TokenStream {
        match self {
            Type::String => quote::quote! { String },
            Type::Bool => quote::quote! { bool },
            Type::Integer(ident) => quote::quote! { #ident },
            Type::Float(ident) => quote::quote! { #ident },
            Type::Object(ident) => quote::quote! { #ident },
            Type::Enum(ident) => quote::quote! { #ident },
            Type::List(ident) => {
                let nested_token_stream = ident.get_type_token_stream();
                quote::quote! { Vec<#nested_token_stream> }
            }
            Type::Map(ident) => {
                let nested_token_stream = ident.get_type_token_stream();
                quote::quote! { std::collections::HashMap<String, #nested_token_stream> }
            }
        }
    }

    pub fn get_nested_pattern(&self, named: bool, nested_ident: Ident) -> TokenStream {
        match (self, named) {
            (Type::String, true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(#nested_ident)
                )
            },
            (Type::String, false) => quote::quote! {
                syn::NestedMeta::Lit(#nested_ident)
            },
            (Type::Bool, true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(#nested_ident)
                )
            },
            (Type::Bool, false) => quote::quote! {
                syn::NestedMeta::Lit(#nested_ident)
            },
            (Type::Integer(_), true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(#nested_ident)
                )
            },
            (Type::Integer(_), false) => quote::quote! {
                syn::NestedMeta::Lit(#nested_ident)
            },
            (Type::Float(_), true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(#nested_ident)
                )
            },
            (Type::Float(_), false) => quote::quote! {
                syn::NestedMeta::Lit(#nested_ident)
            },
            (Type::Object(_), true) => quote::quote! {
                syn::NestedMeta::Meta(#nested_ident)
            },
            (Type::Enum(_), true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::NameValue(#nested_ident)
                )
            },
            (Type::Enum(_), false) => quote::quote! {
                syn::NestedMeta::Lit(#nested_ident)
            },
            (Type::List(_), true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(#nested_ident)
                )
            },
            (Type::Map(_), true) => quote::quote! {
                syn::NestedMeta::Meta(
                    syn::Meta::List(#nested_ident)
                )
            },
            _ => unreachable!(),
        }
    }

    pub fn get_lit_reader_token_stream(
        &self,
        nested_ident: Ident,
        nested_lit: TokenStream,
        path: TokenStream,
        meta_list: Ident,
    ) -> TokenStream {
        match self {
            Type::String => quote::quote! {
                yui::get_lit_str(&#nested_lit, &#path)
            },
            Type::Bool => quote::quote! {
                yui::get_lit_bool(&#nested_lit, &#path)
            },
            Type::Integer(_) => {
                let result_type = self.get_type_token_stream();
                quote::quote! {
                    yui::get_lit_int::<#result_type, String>(&#nested_lit, &#path)
                }
            }
            Type::Float(_) => {
                let result_type = self.get_type_token_stream();
                quote::quote! {
                    yui::get_lit_float::<#result_type, String>(&#nested_lit, &#path)
                }
            }
            Type::Object(_) => {
                let result_type = self.get_type_token_stream();
                quote::quote! {
                    #result_type::from_meta(&#meta_list)
                }
            }
            Type::Enum(_) => {
                quote::quote! {
                    yui::get_lit_str(&#nested_lit, &#path)?.parse().map_err(
                        |e: yui::Error| {
                            syn::Error::new_spanned(&#nested_lit, e.get_message())
                        }
                    )
                }
            }
            Type::List(ty) => {
                let result_type = ty.get_type_token_stream();
                let list_nested_ident = format_ident! {"list_{}", nested_ident};
                let pattern = ty.get_nested_pattern(false, list_nested_ident.clone());
                let reader = ty.get_lit_reader_token_stream(
                    list_nested_ident.clone(),
                    quote::quote! {#list_nested_ident},
                    path.clone(),
                    list_nested_ident.clone(),
                );
                quote::quote! {
                    #nested_ident.nested.iter().map(|meta_nested_meta| {
                        match &meta_nested_meta {
                            #pattern => #reader,
                            _ => Err(syn::Error::new_spanned(
                                &meta_nested_meta,
                                "Only support List of Lit"
                            ))
                        }
                    }).collect::<Result<Vec<#result_type>, syn::Error>>()
                }
            }
            Type::Map(ty) => {
                let result_type = ty.get_type_token_stream();
                let map_nested_ident = format_ident! {"map_{}", nested_ident};
                let pattern = ty.get_nested_pattern(true, map_nested_ident.clone());
                let reader = ty.get_lit_reader_token_stream(
                    map_nested_ident.clone(),
                    quote::quote! { #map_nested_ident.lit },
                    path.clone(),
                    map_nested_ident.clone(),
                );
                quote::quote! {
                    {
                        let value_pairs: Result<Vec<(String, #result_type)>, syn::Error> =
                            #nested_ident.nested.iter().map(|meta_nested_meta| {
                                match &meta_nested_meta {
                                    #pattern => {
                                        Ok((
                                             format!("{}",yui::unwrap_punctuated_first(
                                                    &#map_nested_ident.path.segments,
                                                    syn::Error::new_spanned(
                                                        &#map_nested_ident,
                                                        "Unexpected type path segment"
                                                    )
                                                 )?.ident
                                             ),
                                             #reader?
                                        ))
                                    },
                                    _ => Err(syn::Error::new_spanned(
                                        &meta_nested_meta,
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

    pub fn get_path_ident(&self, nested_ident: Ident) -> TokenStream {
        match self {
            Type::Object(_) => quote::quote! {
                (match &#nested_ident {
                    syn::Meta::List(object_meta_list) => Ok(&object_meta_list.path),
                    _ => Err(syn::Error::new_spanned(&#nested_ident, "Nested value must be List"))
                }?)
            },
            _ => quote::quote! {#nested_ident.path},
        }
    }

    pub fn to_token_token_stream(&self, value: TokenStream, value_name: Ident, is_option: bool) -> TokenStream {
        match self {
            Type::List(nested_type_box) => {
                let nested_type = nested_type_box.as_ref();
                let nested_value_token = nested_type.to_token_token_stream(
                    quote::quote! { nested_value },
                    format_ident!("{}_nested", value_name),
                    false
                );
                let temp_value_name = format_ident!("nested_value_tokens_{}", value_name);
                let temp_value_name_string = temp_value_name.to_string().clone();
                let nested_value_tokens_interpolated = InterpolatedList::new(
                    temp_value_name_string.as_str(),
                    Some(',')
                );

                match is_option {
                    true => quote::quote!{
                        match &#value {
                            Some(value) => {{
                                let #temp_value_name: Vec<proc_macro2::TokenStream> = value.iter().map(
                                    |nested_value| {
                                        #nested_value_token
                                    }
                                ).collect();
                                quote::quote!{Some(vec![#nested_value_tokens_interpolated])}
                            }},
                            None => quote::quote!{ None }
                        }

                    },
                    false => quote::quote!{{
                        let #temp_value_name: Vec<proc_macro2::TokenStream> = #value.iter().map(
                            |nested_value| {
                                #nested_value_token
                            }
                        ).collect();

                        quote::quote!{vec![#nested_value_tokens_interpolated]}
                    }}
                }
            },
            Type::Map(nested_type_box) => {
                let nested_type = nested_type_box.as_ref();
                let nested_value_tokens = nested_type.to_token_token_stream(
                    quote::quote! { nested_value },
                    format_ident!("{}_nested", value_name),
                    false
                );
                let key_interpolated = Interpolated::new("key");

                let temp_value_name = format_ident!("nested_value_tokens_{}", value_name);
                let temp_value_name_string = temp_value_name.to_string().clone();
                let temp_value_name_interpolated = InterpolatedList::new(
                    temp_value_name_string.as_str(),
                    Some(',')
                );
                let nested_value_token_interpolated = Interpolated::new("nested_value_token");

                match is_option {
                    true => quote::quote! {
                        match &#value {
                            Some(value) => {
                                let #temp_value_name: Vec<proc_macro2::TokenStream> = value.iter().map(
                                    |(key, nested_value)| {
                                        let nested_value_token = #nested_value_tokens;
                                        quote::quote! {
                                            (#key_interpolated, #nested_value_token_interpolated)
                                        }
                                    }
                                ).collect();

                                quote::quote! {
                                    Some([
                                        #temp_value_name_interpolated
                                    ]..iter().cloned().collect())
                                }
                            },
                            None => quote::quote!{ None }
                        }
                    },
                    false => quote::quote! {{
                        let #temp_value_name: Vec<proc_macro2::TokenStream> = #value.iter().map(
                            |(key, nested_value)| {
                                let nested_value_token = #nested_value_tokens;
                                quote::quote! {
                                    (String::from(#key_interpolated), #nested_value_token_interpolated)
                                }
                            }
                        ).collect();

                        quote::quote! {
                            [
                                #temp_value_name_interpolated
                            ].iter().cloned().collect()
                        }
                    }}
                }
            }
            Type::String => {
                let temp_value = format_ident!("temp_value_{}", value_name);
                let temp_value_string = temp_value.to_string().clone();
                let temp_value_interpolated = Interpolated::new(
                    temp_value_string.as_str()
                );
                let value_interpolated = Interpolated::new("value");

                match is_option {
                    true => quote::quote! {
                        match &#value {
                            Some(value) => quote::quote! {Some(String::from(#value_interpolated))},
                            None => quote::quote!{None}
                        }
                    },
                    false => quote::quote! {{
                        let #temp_value = #value;
                        quote::quote! {String::from(#temp_value_interpolated)}
                    }}
                }
            },
            _ => {
                let temp_value = format_ident!("temp_value_{}", value_name);
                let temp_value_string = temp_value.to_string().clone();
                let temp_value_interpolated = Interpolated::new(
                    temp_value_string.as_str()
                );
                let value_interpolated = Interpolated::new("value");

                match is_option {
                    true => quote::quote! {
                        match &#value {
                            Some(value) => quote::quote! {Some(#value_interpolated)},
                            None => quote::quote!{None}
                        }
                    },
                    false => quote::quote! {{
                        let #temp_value = #value;
                        quote::quote! {#temp_value_interpolated}
                    }}
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

    pub fn to_token_token_stream(&self, value: TokenStream, value_name: Ident) -> TokenStream {
        self.unwrap().to_token_token_stream(
            value,
            value_name,
            !self.is_required()
        )
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
