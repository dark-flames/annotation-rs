use super::symbol::*;
use super::{helper::get_lit_str, Fields};
use crate::helper::unwrap_punctuated_first;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Ident, Meta, NestedMeta};

pub struct Attribute {
    ident: Ident,
    path: String,
    fields: Fields,
}

impl Attribute {
    pub fn from_ast(input: &DeriveInput) -> Result<Self, Error> {
        match &input.data {
            Data::Struct(data_struct) => {
                let mut path = input.ident.to_string().clone();

                if !input.attrs.is_empty() {
                    for attr in &input.attrs {
                        if attr.path == ATTRIBUTE {
                            path = match attr.parse_meta()? {
                                Meta::List(list) => match unwrap_punctuated_first(
                                    &list.nested,
                                    Error::new_spanned(&list, "Unexpected nested meta"),
                                )? {
                                    NestedMeta::Lit(lit) => get_lit_str(&lit, &attr.path),
                                    _ => Err(Error::new_spanned(
                                        attr,
                                        "Meta of Attribute must be Lit List",
                                    )),
                                },
                                meta => Err(Error::new_spanned(&meta, "Unexpected attribute")),
                            }?
                        }
                    }
                }

                Ok(Attribute {
                    ident: input.ident.clone(),
                    path,
                    fields: Fields::from_ast(&data_struct.fields)?,
                })
            }
            _ => Err(Error::new_spanned(&input, "Attribute must be a struct")),
        }
    }

    pub fn get_reader(&self) -> TokenStream {
        let name = self.ident.clone();
        let parse = self.fields.get_parse_token_stream();
        let construct = self.fields.get_construct_token_stream();
        let path = self.path.clone();

        quote! {
            impl yui::Parse for #name {
                fn get_path() -> yui::Symbol {
                    yui::Symbol::new(#path)
                }

                fn from_meta_list(
                    input: &syn::MetaList
                ) -> Result<Self, syn::Error>
                where
                    Self: std::marker::Sized {
                    #parse

                    Ok(#name #construct)
                }
            }
        }
    }
}
