use super::ty::{DefaultValue, FieldType};

use crate::reader::Interpolated;
use helpers::{get_lit_as_string, get_lit_bool, get_lit_str, Symbol};
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{
    Attribute as SynAttribute, Error, Field as SynField, Fields as SynFields, Ident, Index, Meta,
    NestedMeta,
};

struct FieldAttribute {
    pub path: Option<String>,
    pub enum_value: Option<bool>,
    pub default: Option<String>,
}

trait ValuedField {
    fn get_attribute(attrs: &[SynAttribute]) -> Result<FieldAttribute, Error> {
        let mut attribute = FieldAttribute {
            path: None,
            enum_value: None,
            default: None,
        };
        for attr in attrs.iter() {
            if attr.path == Symbol::new("attribute_field") {
                match &attr.parse_meta()? {
                    Meta::List(list) => {
                        for nested_item in &list.nested {
                            match nested_item {
                                NestedMeta::Meta(Meta::NameValue(path))
                                    if (path.path == Symbol::new("alias")) =>
                                {
                                    attribute.path = Some(get_lit_str(
                                        &path.lit,
                                        &path.path.get_ident().unwrap(),
                                    )?);
                                }
                                NestedMeta::Meta(Meta::NameValue(enum_value))
                                    if (enum_value.path == Symbol::new("enum_value")) =>
                                {
                                    attribute.enum_value = Some(get_lit_bool(
                                        &enum_value.lit,
                                        &enum_value.path.get_ident().unwrap(),
                                    )?);
                                }
                                NestedMeta::Meta(Meta::NameValue(default))
                                    if (default.path == Symbol::new("default")) =>
                                {
                                    attribute.default = Some(get_lit_as_string(
                                        &default.lit,
                                        &default.path.get_ident().unwrap(),
                                    )?);
                                }
                                _ => {
                                    return Err(Error::new_spanned(
                                        nested_item,
                                        "Unexpected nested meta",
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            attr,
                            "The meta of attribute_field must be a List",
                        ));
                    }
                }
            }
        }

        Ok(attribute)
    }

    fn get_temp_var_name(&self) -> Ident;

    fn field_nested_type(&self) -> TokenStream;

    fn get_default(&self) -> &Option<DefaultValue>;

    fn get_temp_var_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let field_nested_type = self.field_nested_type();
        let default_token = match self.get_default() {
            Some(value) => {
                let value_string = value.to_string();
                quote::quote! {Some(#value_string.parse().unwrap())}
            }
            None => quote::quote! {None},
        };
        quote::quote! {
            let mut #temp_var_name: Option<#field_nested_type> = #default_token
        }
    }

    fn get_parse_token_stream(&self) -> TokenStream;

    fn get_construct_token_stream(&self) -> TokenStream;
}

pub struct NamedField {
    name: Ident,
    path: String,
    default: Option<DefaultValue>,
    field_type: FieldType,
}

impl ValuedField for NamedField {
    fn get_temp_var_name(&self) -> Ident {
        self.name.clone()
    }

    fn field_nested_type(&self) -> TokenStream {
        self.field_type.unwrap().get_type_token_stream()
    }

    fn get_default(&self) -> &Option<DefaultValue> {
        &self.default
    }

    fn get_parse_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let path_name = self.path.as_str();
        let nested_ident = format_ident!("nested_{}", temp_var_name);
        let nested_pattern = self
            .field_type
            .unwrap()
            .get_nested_pattern(true, &nested_ident);
        let nested_lit = quote::quote! { #nested_ident.lit };
        let path = quote::quote! { String::from(#path_name) };
        let reader = self.field_type.unwrap().get_lit_reader(
            &nested_ident,
            &nested_lit,
            &path,
            &nested_ident,
        );

        let path_ident = self.field_type.unwrap().get_path_ident(nested_ident);
        quote::quote! {
            #nested_pattern if #path_ident == annotation_rs::Symbol::new(#path_name) => {
                #temp_var_name = Some(#reader?);
            }
        }
    }

    fn get_construct_token_stream(&self) -> TokenStream {
        let field_name = self.name.clone();
        let temp_var_name = self.get_temp_var_name();
        match self.field_type.is_required() {
            true => quote::quote! {
                #field_name: match #temp_var_name {
                    Some(value) => Ok(value),
                    None => Err(syn::Error::new_spanned(
                        quote::format_ident!{"Attribute"},
                        "Not found value on required field"
                    ))
                }?
            },
            false => quote::quote! {
                #temp_var_name
            },
        }
    }
}

impl NamedField {
    pub fn from_ast(input: &SynField) -> Result<Self, Error> {
        let attribute = Self::get_attribute(&input.attrs)?;
        let mut path = input.ident.as_ref().unwrap().to_string();
        if attribute.path.is_some() {
            path = attribute.path.unwrap();
        }

        let mut is_enum = false;
        if attribute.enum_value.is_some() {
            is_enum = attribute.enum_value.unwrap()
        }
        let field_type = FieldType::from_ast(&input.ty, is_enum)?;
        let mut default: Option<DefaultValue> = None;
        if attribute.default.is_some() {
            default = Some(DefaultValue::from_string(
                attribute.default.unwrap(),
                input,
                &field_type.unwrap(),
            )?)
        }
        Ok(NamedField {
            name: input.ident.as_ref().unwrap().clone(),
            path,
            default,
            field_type,
        })
    }
}

pub struct UnnamedFiled {
    index: usize,
    field_type: FieldType,
    default: Option<DefaultValue>,
}

impl ValuedField for UnnamedFiled {
    fn get_temp_var_name(&self) -> Ident {
        format_ident!("temp_{:0}", self.index)
    }

    fn field_nested_type(&self) -> TokenStream {
        self.field_type.unwrap().get_type_token_stream()
    }

    fn get_default(&self) -> &Option<DefaultValue> {
        &self.default
    }

    fn get_parse_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let nested_ident = format_ident!("nested_{}", temp_var_name);
        let nested_pattern = self
            .field_type
            .unwrap()
            .get_nested_pattern(false, &nested_ident);
        let index = self.index;
        let lit_name = format!(
            "{} field",
            match index + 1 {
                1 => String::from("1st"),
                2 => String::from("2nd"),
                3 => String::from("3rd"),
                others => format!("{}th", others),
            },
        );

        let nested_lit = quote::quote! { #nested_ident };
        let path = quote::quote! { String::from(#lit_name) };
        let reader = self.field_type.unwrap().get_lit_reader(
            &nested_ident,
            &nested_lit,
            &path,
            &nested_ident,
        );

        quote::quote! {
            #nested_pattern if field_index == #index => {
                #temp_var_name = Some(#reader?);
            }
        }
    }

    fn get_construct_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        match self.field_type.is_required() {
            true => quote::quote! {
                match #temp_var_name {
                    Some(value) => Ok(value),
                    None => Err(syn::Error::new_spanned(
                        quote::format_ident!{"Attribute"},
                        "Not found value on required field"
                    ))
                }?
            },
            false => quote::quote! {
                #temp_var_name
            },
        }
    }
}

impl UnnamedFiled {
    pub fn from_ast(input: &SynField, index: usize) -> Result<Self, Error> {
        let attribute = Self::get_attribute(&input.attrs)?;

        let mut is_enum = false;
        if attribute.enum_value.is_some() {
            is_enum = attribute.enum_value.unwrap()
        }
        let field_type = FieldType::from_ast(&input.ty, is_enum)?;
        let mut default: Option<DefaultValue> = None;
        if attribute.default.is_some() {
            default = Some(DefaultValue::from_string(
                attribute.default.unwrap(),
                input,
                &field_type.unwrap(),
            )?)
        }
        Ok(UnnamedFiled {
            index,
            default,
            field_type,
        })
    }
}

pub enum Fields {
    NamedFields(Vec<NamedField>),
    UnnamedField(Vec<UnnamedFiled>),
    None,
}

impl Fields {
    pub fn from_ast(fields: &SynFields) -> Result<Self, Error> {
        match fields {
            SynFields::Named(named_fields) => Ok(Fields::NamedFields(
                named_fields
                    .named
                    .iter()
                    .map(|field| NamedField::from_ast(field))
                    .collect::<Result<Vec<NamedField>, Error>>()?,
            )),
            SynFields::Unnamed(unnamed_fields) => {
                let mut fields = Vec::new();

                for (index, field) in unnamed_fields.unnamed.iter().enumerate() {
                    fields.push(UnnamedFiled::from_ast(field, index)?);
                }

                Ok(Fields::UnnamedField(fields))
            }
            SynFields::Unit => Ok(Fields::None),
        }
    }

    pub fn parse_attributes_args_token_stream(
        &self,
        attributes_args_ident: Ident,
        name: Ident,
    ) -> TokenStream {
        let temp_var_token_stream = match &self {
            Fields::NamedFields(fields) => fields
                .iter()
                .map(|field| field.get_temp_var_token_stream())
                .collect(),
            Fields::UnnamedField(fields) => fields
                .iter()
                .map(|field| field.get_temp_var_token_stream())
                .collect(),
            _ => Vec::new(),
        };

        let parse_token_stream = match &self {
            Fields::NamedFields(fields) => fields
                .iter()
                .map(|field| field.get_parse_token_stream())
                .collect(),
            Fields::UnnamedField(fields) => fields
                .iter()
                .map(|field| field.get_parse_token_stream())
                .collect(),
            _ => Vec::new(),
        };

        let construct = self.construct_token_stream(&name);

        let index_value = match self {
            Fields::UnnamedField(_) => quote::quote! {field_index},
            _ => quote::quote! {_},
        };

        match &self {
            Fields::NamedFields(_) | Fields::UnnamedField(_) => {
                quote::quote! {
                    #(#temp_var_token_stream;)*

                    for (#index_value, nested) in #attributes_args_ident.iter().enumerate() {
                        match &nested {
                            #(#parse_token_stream),*
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    nested,
                                    "Unexpected nested value in list"
                                ))
                            }
                        }
                    };

                    #construct
                }
            }
            _ => quote::quote! {
                panic!("No Field attribute structure can not parse from attribute args");
            },
        }
    }

    pub fn construct_token_stream(&self, name: &Ident) -> TokenStream {
        match self {
            Fields::NamedFields(named_fields) => {
                let fields_token_stream: Vec<TokenStream> = named_fields
                    .iter()
                    .map(|field| field.get_construct_token_stream())
                    .collect();
                quote::quote! {
                    Ok(#name {
                        #(#fields_token_stream),*
                    })
                }
            }
            Fields::UnnamedField(unnamed_field) => {
                let fields_token_stream: Vec<TokenStream> = unnamed_field
                    .iter()
                    .map(|field| field.get_construct_token_stream())
                    .collect();
                quote::quote! {
                    (
                        Ok(#name (
                            #(#fields_token_stream),*
                        ))
                    )
                }
            }
            Fields::None => {
                quote::quote! {Ok(#name)}
            }
        }
    }

    pub fn parse_meta_token_stream(&self, name: &Ident) -> TokenStream {
        match self {
            Fields::NamedFields(_) | Fields::UnnamedField(_) => {
                quote::quote! {
                    let input_meta_list = match input {
                        syn::Meta::List(list) => Ok(list),
                        _ => Err(syn::Error::new_spanned(
                            &input,
                            "Argument of attribute must be a List"
                        ))
                    }?;

                    let mut attribute_args: syn::AttributeArgs = syn::AttributeArgs::new();
                    for nested in input_meta_list.nested.clone() {
                        attribute_args.push(nested)
                    }

                    Self::from_attribute_args(attribute_args)
                }
            }
            Fields::None => {
                let construct = self.construct_token_stream(&name);
                quote::quote! {
                    #construct
                }
            }
        }
    }

    pub fn get_to_token_temp_value_token_stream(&self) -> Vec<TokenStream> {
        match &self {
            Fields::NamedFields(fields) => fields
                .iter()
                .map(|field| {
                    let value_name = field.get_temp_var_name();
                    let field_name = field.name.clone();
                    let value_token = field.field_type.to_token(
                        quote::quote! {
                            self.#field_name.clone()
                        },
                        value_name.clone(),
                    );

                    quote::quote! {
                        let #value_name = #value_token
                    }
                })
                .collect(),
            Fields::UnnamedField(fields) => fields
                .iter()
                .map(|field| {
                    let value_name = field.get_temp_var_name();
                    let index = Index::from(field.index);
                    let value_token = field.field_type.to_token(
                        quote::quote! {
                            self.#index.clone()
                        },
                        value_name.clone(),
                    );

                    quote::quote! {
                        let #value_name = #value_token
                    }
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn get_to_token_token_stream(&self, name: TokenStream) -> TokenStream {
        match self {
            Fields::None => quote::quote! {
                #name
            },
            Fields::NamedFields(fields) => {
                let field_tokens: Vec<TokenStream> = fields
                    .iter()
                    .map(|field| {
                        let field_name = field.name.clone();
                        let temp_value_str = field.get_temp_var_name().to_string();
                        let temp_value = temp_value_str.as_str();
                        let temp_value_interpolated = Interpolated::new(temp_value);
                        quote::quote! {
                            #field_name: #temp_value_interpolated
                        }
                    })
                    .collect();
                quote::quote! {
                    #name {
                        #(#field_tokens),*
                    }
                }
            }
            Fields::UnnamedField(fields) => {
                let field_tokens: Vec<TokenStream> = fields
                    .iter()
                    .map(|field| {
                        let temp_value_str = field.get_temp_var_name().to_string();
                        let temp_value = temp_value_str.as_str();
                        let temp_value_interpolated = Interpolated::new(temp_value);
                        quote::quote! {
                            #temp_value_interpolated
                        }
                    })
                    .collect();
                quote::quote! {
                    #name (
                        #(#field_tokens),*
                    )
                }
            }
        }
    }
}
