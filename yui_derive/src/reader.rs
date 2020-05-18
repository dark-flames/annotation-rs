use proc_macro2::Ident;
use syn::parse::{Parse, ParseBuffer};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Comma};
use syn::{bracketed, Error, Token};

pub struct ReaderConfig {
    pub name: Ident,
    pub attr_idents: Punctuated<Ident, Token![,]>,
    pub field_attr_idents: Punctuated<Ident, Token![,]>,
}

#[inline]
fn parse_punctuated_inside_bracket<T: Parse, U: Parse>(
    content: &'a ParseBuffer,
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
    fn parse(input: &'a ParseBuffer<'a>) -> Result<Self, Error> {
        let name = input.parse()?;
        input.parse::<Comma>()?;
        let attr_idents = parse_punctuated_inside_bracket(&input)?;
        input.parse::<Comma>()?;
        let field_attr_idents = parse_punctuated_inside_bracket(&input)?;
        Ok(ReaderConfig {
            name,
            attr_idents,
            field_attr_idents,
        })
    }
}
