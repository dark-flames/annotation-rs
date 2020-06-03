use heck::SnakeCase;
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream};
use quote::{format_ident, ToTokens, TokenStreamExt};
use std::collections::HashSet;
use syn::parse::{Parse, ParseBuffer};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Comma};
use syn::{bracketed, Error, Token};

#[derive(Clone, Copy)]
pub struct Interpolated<'a>(&'a str);

impl ToTokens for Interpolated<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Punct::new('#', Spacing::Alone));
        tokens.append(Ident::new(self.0, Span::call_site()));
    }
}

impl Interpolated<'_> {
    pub fn new(name: &str) -> Interpolated {
        Interpolated(name)
    }
}

#[derive(Clone, Copy)]
pub struct InterpolatedList<'a>(Interpolated<'a>, Option<char>);

impl InterpolatedList<'_> {
    pub fn new(name: &str, punctuate: Option<char>) -> InterpolatedList {
        InterpolatedList(Interpolated(name), punctuate)
    }
}

impl ToTokens for InterpolatedList<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Punct::new('#', Spacing::Alone));
        tokens.append(Group::new(
            Delimiter::Parenthesis,
            self.0.into_token_stream(),
        ));
        if self.1.is_some() {
            tokens.append(Punct::new(self.1.unwrap(), Spacing::Alone));
        }
        tokens.append(Punct::new('*', Spacing::Alone));
    }
}

type AttributeIdents = Punctuated<Ident, Token![,]>;

pub fn idents_to_vec(idents: &AttributeIdents) -> Vec<Ident> {
    idents.iter().map(|ident| ident.clone()).collect()
}

pub struct ReaderConfig {
    pub name: Ident,
    pub attr_idents: AttributeIdents,
    pub field_attr_idents: AttributeIdents,
}

#[inline]
fn parse_punctuated_inside_bracket<T: Parse, U: Parse>(
    content: &ParseBuffer,
) -> Result<Punctuated<T, U>, Error> {
    if content.peek(Bracket) {
        let bracket_content;
        bracketed!(bracket_content in content);
        bracket_content.parse_terminated(T::parse)
    } else {
        Err(content.error("must inside a bracket"))
    }
}

impl Parse for ReaderConfig {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self, Error> {
        let name = input.parse()?;
        input.parse::<Comma>()?;
        let attr_idents = parse_punctuated_inside_bracket(&input)?;
        let field_attr_idents = match input.parse::<Comma>() {
            Ok(_) => parse_punctuated_inside_bracket(&input)?,
            Err(_) => Punctuated::new(),
        };
        Ok(ReaderConfig {
            name,
            attr_idents,
            field_attr_idents,
        })
    }
}

impl ReaderConfig {
    fn get_attribute(&self) -> TokenStream {
        let name = self.name.clone();
        let attributes_hash_set: HashSet<Ident> = [
            self.attr_idents
                .iter()
                .map(|ident| ident.clone())
                .collect::<Vec<Ident>>(),
            self.field_attr_idents
                .iter()
                .map(|ident| ident.clone())
                .collect::<Vec<Ident>>(),
        ]
        .concat()
        .iter()
        .cloned()
        .collect();

        let attributes: Vec<&Ident> = attributes_hash_set.iter().collect();

        quote::quote! {
            #[proc_macro_derive(#name, attributes(#(#attributes),*))]
        }
    }

    fn attributes_reader_token_stream(attribute_map: Vec<Ident>) -> TokenStream {
        let structure_interpolated = Interpolated::new("structure");
        let fn_name_interpolated = Interpolated("fn_name");

        let attribute_matches: Vec<TokenStream> = attribute_map
            .iter()
            .map(|ident| {
                let attribute_name = ident.to_string();
                let snake_case_attribute_name = attribute_name.to_snake_case();
                quote::quote! {
                    Ok(meta) if attr.path == #ident::get_path() => {
                        let fn_name = match &prefix {
                            Some(prefix_name) => quote::format_ident!(
                                "__attr_{}_{}",
                                prefix_name.as_str().to_lowercase(),
                                #snake_case_attribute_name
                            ),
                            None => quote::format_ident!(
                                "__attr_{}",
                                #snake_case_attribute_name
                            )
                        };
                        match #ident::from_meta(&meta) {
                            Ok(structure) => {
                                attribute_map.insert(#attribute_name);
                                Some(Ok(quote::quote! {
                                    pub fn #fn_name_interpolated() -> #ident {#structure_interpolated}
                                }))
                            },
                            Err(e) => Some(Err(e))
                        }
                    }
                }
            }).collect();
        let count_interpolated = Interpolated::new("count");
        let attributes_interpolated = InterpolatedList::new("attributes", Some(','));
        let attribute_map_const_name_interpolated = Interpolated::new("attribute_map_const_name");
        quote::quote! {
            |prefix: Option<String>, attributes: &Vec<syn::Attribute>| -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
                let mut attribute_map: std::collections::HashSet<&str> = std::collections::HashSet::new();

                let attribute_map_const_name = match &prefix {
                    Some(prefix_name) => quote::format_ident!(
                        "{}_ATTRIBUTE_MAP",
                        prefix_name.to_uppercase()
                    ),
                    None => quote::format_ident!(
                        "ATTRIBUTE_MAP"
                    )
                };

                attributes.iter().map(
                    |attr| -> Option<Result<proc_macro2::TokenStream, syn::Error>> {
                        let meta = attr.parse_meta();

                        match meta {
                            #(#attribute_matches,)*
                            Err(e) => Some(Err(e)),
                            _ => None
                        }
                    }
                )
                .filter_map(|result| result)
                .collect::<Result<Vec<proc_macro2::TokenStream>, syn::Error>>().map(
                    |attribute_tokens| {
                        let count = attribute_map.len();
                        let attributes: Vec<_> = attribute_map.into_iter().collect();
                        let tokens = vec![
                            quote::quote!{
                                const #attribute_map_const_name_interpolated: [&'static str; #count_interpolated] = [#attributes_interpolated];
                            }
                        ];

                        [tokens, attribute_tokens].concat()
                    }
                )
            }
        }
    }

    fn read_field_attributes_token_stream(attribute_map: Vec<Ident>) -> TokenStream {
        let attributes_reader = Self::attributes_reader_token_stream(attribute_map);
        let tokens_interpolated = InterpolatedList::new("tokens", None);
        quote::quote! {
            let reader = #attributes_reader;
            match input.data {
                syn::Data::Struct(data_struct) => {
                    data_struct.fields.iter().enumerate().map(
                        |(index, field)| {
                            let field_name = match &field.ident {
                                Some(name) => name.to_string(),
                                None => index.to_string()
                            };

                            let tokens = match reader(Some(field_name), &field.attrs)  {
                                Ok(result) => result,
                                Err(e) => return proc_macro2::TokenStream::from(e.to_compile_error())
                            };

                            quote::quote!{
                                #tokens_interpolated
                            }
                        }
                    ).collect::<Vec<proc_macro2::TokenStream>>()
                },
                syn::Data::Enum(data_enum) => {
                    data_enum.variants.iter().map(
                        |field| {
                            let variant = field.ident.to_string();

                            let tokens = match reader(Some(variant), &field.attrs) {
                                Ok(result) => result,
                                Err(e) => return proc_macro2::TokenStream::from(e.to_compile_error())
                            };

                            quote::quote!{
                                #tokens_interpolated
                            }
                        }
                    ).collect::<Vec<proc_macro2::TokenStream>>()
                },
                syn::Data::Union(data_union) => {
                    data_union.fields.named.iter().map(
                        |field| {
                            let field_name = field.ident.as_ref().unwrap().clone().to_string();

                            let tokens = match reader(Some(field_name), &field.attrs)  {
                                Ok(result) => result,
                                Err(e) => return proc_macro2::TokenStream::from(e.to_compile_error())
                            };

                            quote::quote!{
                                #tokens_interpolated
                            }
                        }
                    ).collect::<Vec<proc_macro2::TokenStream>>()
                }
            }
        }
    }

    pub fn get_reader(&self) -> TokenStream {
        let attribute = self.get_attribute();
        let fn_name = format_ident!("derive_{}", self.name.to_string().to_snake_case());
        let struct_attribute_reader =
            Self::attributes_reader_token_stream(idents_to_vec(&self.attr_idents));
        let field_attribute_reader =
            Self::read_field_attributes_token_stream(idents_to_vec(&self.field_attr_idents));

        let name_interpolated = Interpolated("name");
        let struct_attribute_tokens_interpolated =
            InterpolatedList::new("struct_attribute_tokens", None);
        let field_attribute_tokens_interpolated =
            InterpolatedList::new("field_attribute_tokens", None);

        quote::quote! {
            #attribute
            pub fn #fn_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                use yui::AttributeStructure;
                let input = syn::parse_macro_input!(input as syn::DeriveInput);

                let name = input.ident;

                let struct_attribute_reader = #struct_attribute_reader;

                let struct_attribute_tokens: Vec<proc_macro2::TokenStream> = match struct_attribute_reader(None, &input.attrs) {
                    Ok(tokens) => tokens,
                    Err(e) => return proc_macro::TokenStream::from(e.to_compile_error())
                };

                let field_attribute_tokens: Vec<proc_macro2::TokenStream> = {
                    #field_attribute_reader
                };

                proc_macro::TokenStream::from(quote::quote! {
                    impl #name_interpolated {
                        #struct_attribute_tokens_interpolated
                        #field_attribute_tokens_interpolated
                    }
                })
            }
        }
    }
}

pub struct GetAttributeParam {
    class: Ident,
    attribute: Ident,
    property: Option<Ident>,
}

impl Parse for GetAttributeParam {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self, Error> {
        let class = input.parse()?;
        input.parse::<Comma>()?;
        let attribute = input.parse()?;
        let property = match input.parse::<Comma>() {
            Ok(_) => input.parse()?,
            Err(_) => None,
        };
        Ok(GetAttributeParam {
            class,
            attribute,
            property,
        })
    }
}

impl GetAttributeParam {
    pub fn get_attribute(&self) -> TokenStream {
        let attr_str = self.attribute.to_string().to_snake_case();
        let fn_name = format_ident!(
            "__attr_{}",
            match &self.property {
                Some(prop) => {
                    let prop_str = prop.to_string().to_lowercase();
                    format_ident!("{}_{}", prop_str, attr_str)
                }
                None => format_ident!("{}", attr_str),
            }
        );
        let class = &self.class;
        let has_attr = self.has_attribute();
        quote::quote! {
            match #has_attr {
                true => Some(#class::#fn_name()),
                false => None
            }
        }
    }

    pub fn has_attribute(&self) -> TokenStream {
        let const_name = format_ident!(
            "{}ATTRIBUTE_MAP",
            match &self.property {
                Some(prop) => {
                    let prop_str = prop.to_string().to_uppercase();
                    format_ident!("{}_", prop_str)
                }
                None => format_ident!(""),
            }
        );
        let attr_str = self.attribute.to_string();
        let class = &self.class;
        quote::quote! {
            #class::#const_name.contains(&#attr_str)
        }
    }
}
