mod attribute;
mod enums;

use crate::attribute::TestNoFieldStruct;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use yui::Attributes;

#[proc_macro_derive(NoField, attributes(TestNoFieldStruct))]
pub fn derive_no_field(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident.clone();
    TokenStream::from(
        match Attributes::<TestNoFieldStruct>::from_derive_input(input) {
            Ok(attributes) => {
                let count = attributes.attrs.len();

                quote! {
                    impl #ident {
                        pub fn count() -> usize {
                            #count
                        }
                    }
                }
            }
            Err(e) => e.to_compile_error(),
        },
    )
}
