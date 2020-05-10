use super::helper::get_lit_str;
use super::symbol::*;
use heck::SnakeCase;
use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Error, Meta, NestedMeta, Variant};

pub struct EnumItem {
    ident: String,
    value: String,
}

impl EnumItem {
    pub fn from_ast(input: &Variant) -> Result<Self, Error> {
        let ident = input.ident.to_string().clone();
        let mut value = ident.to_snake_case();

        if !input.attrs.is_empty() {
            for attr in input.attrs.iter() {
                if name_value.path == ENUM_ITEM_VALUE {
                    match attr.parse_meta() {
                        Meta::List(list) => match list.nested.iter().first() {
                            NestedMeta::Lit(lit) => value = get_lit_str(&lit, &attr.path)?,
                            _ => Err(Error::new_spanned(
                                attr,
                                "Meta of enum_item_value must be Lit List",
                            )),
                        },
                        _ => {
                            return Err(Error::new_spanned(
                                input,
                                "Meta of enum_item_value must be Lit List",
                            ))
                        }
                    };
                }
            }
        }

        Ok(EnumItem { ident, value })
    }

    pub fn to_pattern_token_stream(&self, enum_name: &String) -> TokenStream {
        let item_value = self.value.as_str();
        let item_ident = self.ident.as_str();
        quote! {
            #item_value => Ok(#enum_name::#item_ident)
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
            _ => Err(Error::new_spanned(
                input,
                "EnumValue can only be created from Enum",
            )),
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
            impl yui::traits::ValueEnum #enum_name {
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value {
                        #(#arm),*
                        others => Err(
                            syn::Error::new_spanned(
                                lit,
                                format!("Unexpected {} value: {}", #enum_name, path),
                            )
                        )
                    }
                }
            }
        }
    }
}
