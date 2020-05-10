use super::symbol::*;
use super::{helper::get_lit_bool, helper::get_lit_str, DefaultValue, FieldType};
use proc_macro2::TokenStream;
use quote;
use std::collections::HashMap;
use syn::{Error, Field as SynField, Fields as SynFields, Meta, NestedMeta, Path};

trait ValuedField {
    fn get_attributes(attrs: &Vec<SynAttribute>) -> Result<HashMap<&str, (&Lit, &Path)>, Error> {
        let mut map = HashMap::new();
        for attr in attrs.iter() {
            if attr.path == ATTRIBUTE_FIELD {
                match &attr.parse_meta() {
                    Meta::List(list) => {
                        for nested_item in &list.nested {
                            match nested_item {
                                NestedMeta::Meta(Meta::NameValue(path))
                                    if (path.path == ATTRIBUTE_FIELD_PATH) =>
                                {
                                    map.insert("path", (&path.lit, &path.path))
                                }
                                NestedMeta::Meta(Meta::NameValue(enum_value))
                                    if (enum_value.path == ATTRIBUTE_FIELD_ENUM_VALUE) =>
                                {
                                    map.insert("enum_value", (&enum_value.lit, &enum_value.path))
                                }
                                NestedMeta::Meta(Meta::NameValue(default))
                                    if (default.path == ATTRIBUTE_FIELD_DEFAULT) =>
                                {
                                    map.insert("enum_value", (&default.lit, &default.path))
                                }
                                _ => Err(Error::new_spanned(nested_item, "Unexpected nested meta")),
                            }
                        }
                    }
                    _ => {
                        return Err(Error::new_spanned(
                            attr,
                            "The meta of attribute_field must be a List",
                        ))
                    }
                }
            }
        }

        Ok(map)
    }

    fn get_temp_var_name(&self) -> &str;

    fn field_nested_type(&self) -> &str;

    fn get_default(&self) -> Option<&DefaultValue>;

    fn get_temp_var_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let field_nested_type = self.field_nested_type();
        let default_token = match self.get_default() {
            Some(value) => {
                let value_str = value.to_string().as_str();
                quote! {#value_str.parse()?}
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
    fn get_temp_var_name(&self) -> &str {
        format!("temp_{}", self.name).as_str()
    }

    fn field_nested_type(&self) -> &str {
        self.field_type.unwrap().to_string().as_str()
    }

    fn get_default(&self) -> &Option<DefaultValue> {
        &self.default
    }

    fn get_parse_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let path_name = self.path.as_str();
        let nested_pattern = self.field_type.unwrap().get_nested_pattern(true)?;
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
        let field_name = self.name.as_str();
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
        let attributes = Self::get_attributes(&input.attrs)?;
        let mut path = input.ident.unwrap().to_string().clone();
        if attributes.contains_key("path") {
            let (path_lit, path_path) = attributes["path"];
            path = get_lit_str(path_lit, path_path)?;
        }

        let mut is_enum = false;
        if attributes.contains_key("enum_value") {
            let (enum_value_lit, enum_value_path) = attributes["path"];
            is_enum = get_lit_bool(enum_value_lit, enum_value_path)?
        }
        let field_type = FieldType::from_ast(&input.ty, is_enum)?;
        let mut default: Option<DefaultValue> = None;
        if attributes.contains_key("default") {
            let (default_lit, default_path) = attributes["path"];
            default = Some(DefaultValue::from_lit(
                &default_lit,
                &default_path,
                &field_type.unwrap(),
            )?)
        }
        Ok(NamedField {
            name: input.ident.unwrap().to_string().clone(),
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
    fn get_temp_var_name(&self) -> &str {
        format!("temp_{}", self.index).as_str()
    }

    fn field_nested_type(&self) -> &str {
        self.field_type.unwrap().to_string().as_str()
    }

    fn get_default(&self) -> &Option<DefaultValue> {
        &self.default
    }

    fn get_parse_token_stream(&self) -> TokenStream {
        let temp_var_name = self.get_temp_var_name();
        let path_name = self.path.as_str();
        let nested_pattern = self.field_type.unwrap().get_nested_pattern(false)?;
        let reader =
            self.field_type
                .unwrap()
                .get_lit_reader_token_stream("&lit", "&input.path", "&input");
        quote! {
            #nested_pattern if meta_value.path == yui::Symbol(#path_name) => {
                #temp_var_name = Some(#reader?)
            }
        }
    }

    fn get_construct_token_stream(&self) -> TokenStream {
        let field_name = self.name.as_str();
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
        let mut is_enum = false;
        let attributes = Self::get_attributes(&input.attrs)?;
        if attributes.contains_key("enum_value") {
            if attributes.contains_key("enum_value") {
                let (enum_value_lit, enum_value_path) = attributes["path"];
                is_enum = get_lit_bool(enum_value_lit, enum_value_path)?
            }
        }
        let field_type = FieldType::from_ast(&input.ty, is_enum)?;
        let mut default: Option<DefaultValue> = None;
        if attributes.contains_key("default") {
            let (default_lit, default_path) = attributes["path"];
            default = Some(DefaultValue::from_lit(
                &default_lit,
                &default_path,
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
        match &data_struct.fields {
            SynFields::Named(named_fields) => Ok(Fields::NamedFields(
                named_fields
                    .named
                    .iter()
                    .map(|field| NamedField::from_ast(field))
                    .collect()?,
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
            Fields::NamedFields(fields) | Fields::UnnamedField(fields) => fields
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
