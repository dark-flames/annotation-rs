use super::helper::get_lit_str;
use super::symbol::*;
use crate::helper::unwrap_punctuated_first;
use heck::SnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Meta, NestedMeta, Variant};

pub struct EnumItem {
    ident: Ident,
    value: String,
}

impl EnumItem {
    pub fn from_ast(input: &Variant) -> Result<Self, Error> {
        let ident = input.ident.clone();
        let mut value = ident.to_string().to_snake_case();

        if !input.attrs.is_empty() {
            for attr in input.attrs.iter() {
                if attr.path == ENUM_ITEM_VALUE {
                    match attr.parse_meta()? {
                        Meta::List(list) => match unwrap_punctuated_first(
                            &list.nested,
                            Error::new_spanned(&list, "Unexpected nested segment"),
                        )? {
                            NestedMeta::Lit(lit) => value = get_lit_str(&lit, &attr.path)?,
                            _ => {
                                return Err(Error::new_spanned(
                                    attr,
                                    "Meta of enum_item_value must be Lit List",
                                ))
                            }
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

    pub fn to_pattern_token_stream(&self, enum_name: &Ident) -> TokenStream {
        let item_value = self.value.clone();
        let item_ident = &self.ident;
        quote! {
            #item_value => Ok(#enum_name::#item_ident)
        }
    }
}

pub struct EnumValue {
    ident: Ident,
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
                    ident: input.ident.clone(),
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
        let enum_ident = &self.ident;
        let enum_name = self.ident.to_string();
        let arms: Vec<TokenStream> = self
            .items
            .iter()
            .map(|item| item.to_pattern_token_stream(&self.ident))
            .collect();
        quote! {
            impl std::str::FromStr for #enum_ident {
                type Err = yui::Error;
                fn from_str(value: &str) -> Result<Self, Self::Err> {
                    match value {
                        #(#arms,)*
                        others => Err(
                            yui::Error::new(
                                format!("Unexpected {} value: {}", #enum_name, others),
                            )
                        )
                    }
                }
            }

            impl yui::ValueEnum for #enum_ident {}
        }
    }
}
