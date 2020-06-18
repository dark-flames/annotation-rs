use crate::Symbol;
use syn::parse::{Parse, ParseBuffer};
use syn::{AttributeArgs, DeriveInput, Error, Meta};

pub trait AttributeStructure {
    fn get_path() -> Symbol where Self: Sized;

    fn from_meta(input: &Meta) -> Result<Self, syn::Error>
    where
        Self: std::marker::Sized;

    fn from_attribute_args(input: AttributeArgs) -> Result<Self, syn::Error>
    where
        Self: std::marker::Sized;
}

pub struct AttributeStructures<T: AttributeStructure> {
    pub attrs: Vec<T>,
}

impl<T: AttributeStructure> AttributeStructures<T> {
    pub fn from_derive_input(derive_input: &DeriveInput) -> Result<Self, Error> {
        let attributes: Vec<T> = derive_input
            .attrs
            .iter()
            .map(|attr| match attr.parse_meta() {
                Ok(meta) => T::from_meta(&meta),
                Err(e) => Err(e),
            })
            .collect::<Result<Vec<T>, Error>>()?;

        Ok(AttributeStructures { attrs: attributes })
    }
}

impl<T: AttributeStructure> Parse for AttributeStructures<T> {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let derive_input = DeriveInput::parse(&input)?;
        Self::from_derive_input(&derive_input)
    }
}
