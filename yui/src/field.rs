use super::symbol::*;
use super::{helper::get_lit_bool, helper::get_lit_str, DefaultValue, FieldType};
use crate::helper::get_lit_as_string;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute as SynAttribute, Error, Field as SynField, Fields as SynFields, Meta, NestedMeta,
};

struct FieldAttribute {
    pub path: Option<String>,
    pub eum_value: Option<bool>,
    pub default: Option<String>,
}

trait ValuedField {
    fn get_attribute(attrs: &Vec<SynAttribute>) -> Result<FieldAttribute, Error> {
        let mut attribute = FieldAttribute {
            path: None,
            eum_value: None,
            default: None,
        };
        for attr in attrs.iter() {
            if attr.path == ATTRIBUTE_FIELD {
                match &attr.parse_meta()? {
                    Meta::List(list) => {
                        for nested_item in &list.nested {
                            match nested_item {
                                NestedMeta::Meta(Meta::NameValue(path))
                                    if (path.path == ATTRIBUTE_FIELD_PATH) =>
                                {
                                    attribute.path = Some(get_lit_str(&path.lit, &path.path)?);
                                }
                                NestedMeta::Meta(Meta::NameValue(enum_value))
                                    if (enum_value.path == ATTRIBUTE_FIELD_ENUM_VALUE) =>
                                {
                                    attribute.eum_value =
                                        Some(get_lit_bool(&enum_value.lit, &enum_value.path)?);
                                }
                                NestedMeta::Meta(Meta::NameValue(default))
                                    if (default.path == ATTRIBUTE_FIELD_DEFAULT) =>
                                {
                                    attribute.default =
                                        Some(get_lit_as_string(&default.lit, &default.path)?);
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

    fn get_temp_var_name(&self) -> String;

    fn field_nested_type(&self) -> String;

    fn get_default(&self) -> &Option<DefaultValue>;

    fn get_temp_var_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let field_nested_type = self.field_nested_type();
        let default_token = match self.get_default() {
            Some(value) => {
                let value_string = value.to_string();
                quote! {#value_string.parse()?}
            }
            None => quote! {None},
        };
        quote! {
            let mut #temp_var_name: Option<#field_nested_type> = #default_token
        }
    }

    fn get_parse_token_stream(&self) -> TokenStream;

    fn get_construct_token_stream(&self) -> TokenStream;
}

pub struct NamedField {
    name: String,
    path: String,
    default: Option<DefaultValue>,
    field_type: FieldType,
}

impl ValuedField for NamedField {
    fn get_temp_var_name(&self) -> String {
        format!("temp_{}", self.name)
    }

    fn field_nested_type(&self) -> String {
        self.field_type.unwrap().to_string()
    }

    fn get_default(&self) -> &Option<DefaultValue> {
        &self.default
    }

    fn get_parse_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let path_name = self.path.as_str();
        let nested_pattern = self.field_type.unwrap().get_nested_pattern(true);
        let reader = self.field_type.unwrap().get_lit_reader_token_stream(
            "&meta_value.list",
            "&meta_value.path",
            "&meta_value",
        );
        quote! {
            #nested_pattern if meta_value.path == yui::Symbol(#path_name) => {
                #temp_var_name = Some(#reader?)
            }
        }
    }

    fn get_construct_token_stream(&self) -> TokenStream {
        let field_name = self.name.clone();
        let temp_var_name = self.get_temp_var_name();
        match self.field_type.is_required() {
            true => quote! {
                #field_name: match #temp_var_name {
                    Some(value) => Ok(value),
                    None => Err(syn::Error::new_spanned(
                        input,
                        "Not found value on required field"
                    ))
                }?
            },
            false => quote! {
                #field_name: #temp_var_name
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
        if attribute.eum_value.is_some() {
            is_enum = attribute.eum_value.unwrap()
        }
        let field_type = FieldType::from_ast(&input.ty, is_enum)?;
        let mut default: Option<DefaultValue> = None;
        if attribute.default.is_some() {
            default = Some(DefaultValue::from_string(
                attribute.default.unwrap().clone(),
                input,
                &field_type.unwrap(),
            )?)
        }
        Ok(NamedField {
            name: input.ident.as_ref().unwrap().to_string(),
            path,
            default,
            field_type,
        })
    }
}

pub struct UnnamedFiled {
    index: u16,
    field_type: FieldType,
    default: Option<DefaultValue>,
}

impl ValuedField for UnnamedFiled {
    fn get_temp_var_name(&self) -> String {
        format!("temp_{}", self.index)
    }

    fn field_nested_type(&self) -> String {
        self.field_type.unwrap().to_string()
    }

    fn get_default(&self) -> &Option<DefaultValue> {
        &self.default
    }

    fn get_parse_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let nested_pattern = self.field_type.unwrap().get_nested_pattern(false);
        let reader =
            self.field_type
                .unwrap()
                .get_lit_reader_token_stream("&lit", "&input.path", "&input");
        quote! {
            #nested_pattern => {
                #temp_var_name = Some(#reader?)
            }
        }
    }

    fn get_construct_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        match self.field_type.is_required() {
            true => quote! {
                match #temp_var_name {
                    Some(value) => Ok(value),
                    None => Err(syn::Error::new_spanned(
                        input,
                        "Not found value on required field"
                    ))
                }?
            },
            false => quote! {
                #temp_var_name
            },
        }
    }
}

impl UnnamedFiled {
    pub fn from_ast(input: &SynField, index: u16) -> Result<Self, Error> {
        let attribute = Self::get_attribute(&input.attrs)?;

        let mut is_enum = false;
        if attribute.eum_value.is_some() {
            is_enum = attribute.eum_value.unwrap()
        }
        let field_type = FieldType::from_ast(&input.ty, is_enum)?;
        let mut default: Option<DefaultValue> = None;
        if attribute.default.is_some() {
            default = Some(DefaultValue::from_string(
                attribute.default.unwrap().clone(),
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
                let mut index: u16 = 0;
                let mut fields = Vec::new();

                for field in unnamed_fields.unnamed.iter() {
                    fields.push(UnnamedFiled::from_ast(field, index)?);
                    index += 1;
                }

                Ok(Fields::UnnamedField(fields))
            }
            SynFields::Unit => Ok(Fields::None),
        }
    }
    pub fn get_parse_token_stream(&self) -> TokenStream {
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

        match &self {
            Fields::NamedFields(_) | Fields::UnnamedField(_) => {
                quote! {
                    #(#temp_var_token_stream); *

                    for nested in input.nested.iter() {
                        match &nested {
                            #(#parse_token_stream), *
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    nested,
                                    "Unexpected nested value in list"
                                ))
                            }
                        }
                    }
                }
            }
            _ => quote! {},
        }
    }

    pub fn get_construct_token_stream(&self) -> TokenStream {
        match self {
            Fields::NamedFields(named_fields) => {
                let fields_token_stream: Vec<TokenStream> = named_fields
                    .iter()
                    .map(|field| field.get_construct_token_stream())
                    .collect();
                quote! {
                    {
                        #(#fields_token_stream),*
                    }
                }
            }
            Fields::UnnamedField(unnamed_field) => {
                let fields_token_stream: Vec<TokenStream> = unnamed_field
                    .iter()
                    .map(|field| field.get_construct_token_stream())
                    .collect();
                quote! {
                    (
                        #(#fields_token_stream),*
                    )
                }
            }
            Fields::None => {
                quote! {}
            }
        }
    }
}
