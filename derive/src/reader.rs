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

type AnnotationIdents = Punctuated<Ident, Token![,]>;

pub fn idents_to_vec(idents: &AnnotationIdents) -> Vec<Ident> {
    idents.iter().cloned().collect()
}

pub struct ReaderConfig {
    pub name: Ident,
    pub annotation_idents: AnnotationIdents,
    pub field_attr_idents: AnnotationIdents,
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
        let annotation_idents = parse_punctuated_inside_bracket(&input)?;
        let field_attr_idents = match input.parse::<Comma>() {
            Ok(_) => parse_punctuated_inside_bracket(&input)?,
            Err(_) => Punctuated::new(),
        };
        Ok(ReaderConfig {
            name,
            annotation_idents,
            field_attr_idents,
        })
    }
}

impl ReaderConfig {
    fn get_annotation(&self) -> TokenStream {
        let name = self.name.clone();
        let annotation_hash_set: HashSet<Ident> = [
            self.annotation_idents
                .iter()
                .cloned()
                .collect::<Vec<Ident>>(),
            self.field_attr_idents
                .iter()
                .cloned()
                .collect::<Vec<Ident>>(),
        ]
        .concat()
        .iter()
        .cloned()
        .collect();

        let annotation: Vec<&Ident> = annotation_hash_set.iter().collect();

        quote::quote! {
            #[proc_macro_derive(#name, attributes(#(#annotation),*))]
        }
    }

    fn annotation_reader_token_stream(annotation_map: Vec<Ident>) -> TokenStream {
        let structure_interpolated = Interpolated::new("structure");
        let fn_name_interpolated = Interpolated("fn_name");

        let annotation_matches: Vec<TokenStream> = annotation_map
            .iter()
            .map(|ident| {
                let annotation_name = ident.to_string();
                let snake_case_annotation_name = annotation_name.to_snake_case();
                quote::quote! {
                    Ok(meta) if attr.path == #ident::get_path() => {
                        let fn_name = match &prefix {
                            Some(prefix_name) => quote::format_ident!(
                                "__attr_{}_{}",
                                prefix_name.as_str().to_lowercase(),
                                #snake_case_annotation_name
                            ),
                            None => quote::format_ident!(
                                "__attr_{}",
                                #snake_case_annotation_name
                            )
                        };
                        match #ident::from_meta(&meta) {
                            Ok(structure) => {
                                annotation_map.insert(#annotation_name);
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
        let annotations_interpolated = InterpolatedList::new("annotations", Some(','));
        let annotation_map_const_name_interpolated = Interpolated::new("annotation_map_const_name");
        quote::quote! {
            |prefix: Option<String>, annotations: &Vec<syn::Attribute>| -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
                let mut annotation_map: std::collections::HashSet<&str> = std::collections::HashSet::new();

                let annotation_map_const_name = match &prefix {
                    Some(prefix_name) => quote::format_ident!(
                        "{}_ATTRIBUTE_MAP",
                        prefix_name.to_uppercase()
                    ),
                    None => quote::format_ident!(
                        "ATTRIBUTE_MAP"
                    )
                };

                annotations.iter().map(
                    |attr| -> Option<Result<proc_macro2::TokenStream, syn::Error>> {
                        let meta = attr.parse_meta();

                        match &meta {
                            #(#annotation_matches,)*
                            Err(e) => Some(Err(e.clone())),
                            _ => None
                        }
                    }
                )
                .filter_map(|result| result)
                .collect::<Result<Vec<proc_macro2::TokenStream>, syn::Error>>().map(
                    |annotation_tokens| {
                        let count = annotation_map.len();
                        let annotations: Vec<_> = annotation_map.into_iter().collect();
                        let tokens = vec![
                            quote::quote!{
                                const #annotation_map_const_name_interpolated: [&'static str; #count_interpolated] = [#annotations_interpolated];
                            }
                        ];

                        [tokens, annotation_tokens].concat()
                    }
                )
            }
        }
    }

    fn read_field_annotations_token_stream(annotation_map: Vec<Ident>) -> TokenStream {
        let annotations_reader = Self::annotation_reader_token_stream(annotation_map);
        let tokens_interpolated = InterpolatedList::new("tokens", None);
        quote::quote! {
            let reader = #annotations_reader;
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
        let annotation = self.get_annotation();
        let fn_name = format_ident!("derive_{}", self.name.to_string().to_snake_case());
        let struct_annotation_reader =
            Self::annotation_reader_token_stream(idents_to_vec(&self.annotation_idents));
        let field_annotation_reader =
            Self::read_field_annotations_token_stream(idents_to_vec(&self.field_attr_idents));

        let name_interpolated = Interpolated("name");
        let struct_annotation_tokens_interpolated =
            InterpolatedList::new("struct_annotation_tokens", None);
        let field_annotation_tokens_interpolated =
            InterpolatedList::new("field_annotation_tokens", None);

        quote::quote! {
            #annotation
            pub fn #fn_name(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                use annotation_rs::AnnotationStructure;
                let input = syn::parse_macro_input!(input as syn::DeriveInput);

                let name = input.ident;

                let struct_annotation_reader = #struct_annotation_reader;

                let struct_annotation_tokens: Vec<proc_macro2::TokenStream> = match struct_annotation_reader(None, &input.attrs) {
                    Ok(tokens) => tokens,
                    Err(e) => return proc_macro::TokenStream::from(e.to_compile_error())
                };

                let field_annotation_tokens: Vec<proc_macro2::TokenStream> = {
                    #field_annotation_reader
                };

                proc_macro::TokenStream::from(quote::quote! {
                    impl #name_interpolated {
                        #struct_annotation_tokens_interpolated
                        #field_annotation_tokens_interpolated
                    }
                })
            }
        }
    }
}

pub struct GetAnnotationParam {
    class: Ident,
    annotation: Ident,
    property: Option<Ident>,
}

impl Parse for GetAnnotationParam {
    fn parse<'a>(input: &'a ParseBuffer<'a>) -> Result<Self, Error> {
        let class = input.parse()?;
        input.parse::<Comma>()?;
        let annotation = input.parse()?;
        let property = match input.parse::<Comma>() {
            Ok(_) => input.parse()?,
            Err(_) => None,
        };
        Ok(GetAnnotationParam {
            class,
            annotation,
            property,
        })
    }
}

impl GetAnnotationParam {
    pub fn get_annotation(&self) -> TokenStream {
        let attr_str = self.annotation.to_string().to_snake_case();
        let fn_name = format_ident!(
            "__attr_{}",
            match &self.property {
                Some(prop) => {
                    let prop_str = prop.to_string().to_lowercase();
                    format!("{}_{}", prop_str, attr_str)
                }
                None => attr_str,
            }
        );
        let class = &self.class;
        let has_attr = self.has_annotation();
        quote::quote! {
            match #has_attr {
                true => Some(#class::#fn_name()),
                false => None
            }
        }
    }

    pub fn has_annotation(&self) -> TokenStream {
        let const_name = format_ident!(
            "{}ATTRIBUTE_MAP",
            match &self.property {
                Some(prop) => {
                    let prop_str = prop.to_string().to_uppercase();
                    format!("{}_", prop_str)
                }
                None => format!(""),
            }
        );
        let attr_str = self.annotation.to_string();
        let class = &self.class;
        quote::quote! {
            #class::#const_name.contains(&#attr_str)
        }
    }
}
