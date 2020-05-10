use super::symbol::*;
use super::{helper::get_lit_str, DefaultValue, EnumItem, EnumValue, FieldType, Fields, Type};
use proc_macro2::TokenStream;
use syn::{Data, DeriveInput, Error, Meta, NestedMeta};

pub struct Attribute {
    name: String,
    path: String,
    fields: Fields,
}

impl Attribute {
    pub fn from_ast(input: DeriveInput) -> Result<Self, Error> {
        match input {
            Data::Struct(data_struct) => {
                let mut path = input.ident.to_string().clone();

                if !input.attrs.is_empty() {
                    for attr in &input.attrs {
                        if attr.path == ATTRIBUTE {
                            path = match attr.parse_meta() {
                                Meta::List(list) => match list.nested.iter().first() {
                                    NestedMeta::Lit(lit) => path = get_lit_str(&lit, &attr.path)?,
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
                    name: input.ident.to_string().clone(),
                    path,
                    fields: Fields::from_ast(&data_struct.fields)?,
                })
            }
            _ => Err(Error::new_spanned(&input, "Attribute must be a struct")),
        }
    }

    pub fn get_reader(&self) -> TokenStream {
        let name = self.name.clone();
        let parse = self.fields.get_parse_token_stream();
        let construct = self.fields.get_construct_token_stream();
        let path = self.path.as_str();

        quote! {
            impl Parse for #name {
                pub fn get_path() -> yui::Symbol {
                    return yui::Symbol(#path);
                }

                pub fn from_meta_list(
                    input: &syn::MetaList
                ) -> Result<Self, syn::Error> {
                    #parse

                    Ok(#name #construct)
                }
            }
        }
    }
}
