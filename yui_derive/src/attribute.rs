use crate::field::Fields;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Error, Ident, Meta, NestedMeta};
use yui_internal::{get_lit_str, unwrap_punctuated_first, Symbol};

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
                        if attr.path == Symbol::new("attribute") {
                            path = match attr.parse_meta()? {
                                Meta::List(list) => match unwrap_punctuated_first(
                                    &list.nested,
                                    Error::new_spanned(&list, "Unexpected nested meta"),
                                )? {
                                    NestedMeta::Lit(lit) => {
                                        get_lit_str(&lit, &attr.path.get_ident().unwrap())
                                    }
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

    pub fn get_implement(&self) -> TokenStream {
        let name = self.ident.clone();
        let from_attributes_args = self
            .fields
            .parse_attributes_args_token_stream(format_ident!("input"), name.clone());
        let from_meta = self.fields.parse_meta_token_stream(name.clone());
        let path = self.path.clone();

        quote! {
            impl yui::AttributeStructure for #name {
                fn get_path() -> yui::Symbol {
                    yui::Symbol::new(#path)
                }

                fn from_meta(
                    input: syn::Meta
                ) -> Result<Self, syn::Error>
                where
                    Self: std::marker::Sized {
                    #from_meta
                }

                fn from_attribute_args(input: syn::AttributeArgs) -> Result<Self, syn::Error>
                where
                    Self: std::marker::Sized {
                    #from_attributes_args
                }
            }

            impl syn::parse_macro_input::ParseMacroInput for #name {
                fn parse(input: syn::parse::ParseStream) -> Result<Self, syn::Error> {
                    let attribute_args = syn::AttributeArgs::parse(input)?;

                    Self::from_attribute_args(attribute_args)
                }
            }
        }
    }
}
