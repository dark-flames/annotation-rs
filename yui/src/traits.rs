use crate::Symbol;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::{AttributeArgs, DeriveInput, Error, Meta};

pub trait AttributeStructure {
    fn get_path() -> Symbol;

    fn from_meta(input: Meta) -> Result<Self, syn::Error>
    where
        Self: std::marker::Sized;

    fn from_attribute_args(input: AttributeArgs) -> Result<Self, syn::Error>
    where
        Self: std::marker::Sized;
}

pub struct Attributes<T: AttributeStructure> {
    pub attrs: Vec<T>,
}

impl<T: AttributeStructure> Attributes<T> {
    pub fn from_derive_input(derive_input: DeriveInput) -> Result<Self, Error> {
        let attributes: Vec<T> = derive_input
            .attrs
            .iter()
            .map(|attr| match attr.parse_meta() {
                Ok(meta) => T::from_meta(meta),
                Err(e) => Err(e),
            })
            .collect::<Result<Vec<T>, Error>>()?;

        Ok(Attributes { attrs: attributes })
    }
}

impl<T: AttributeStructure> Parse for Attributes<T> {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let derive_input = DeriveInput::parse(&input)?;

        Self::from_derive_input(derive_input)
    }
}
