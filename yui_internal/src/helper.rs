use std::str::FromStr;
use syn::export::fmt::Display;
use syn::punctuated::Punctuated;
use syn::{Error, GenericArgument, Lit, PathArguments, PathSegment, Type, TypePath};

#[inline]
pub fn unwrap_punctuated_first<T, P>(
    punctuated: &Punctuated<T, P>,
    error: Error,
) -> Result<&T, Error> {
    match punctuated.first() {
        Some(s) => Ok(s),
        None => Err(error),
    }
}

#[inline]
pub fn get_nested_type<'a>(
    segment: &'a PathSegment,
    message: &'static str,
) -> Result<&'a Type, Error> {
    let error = Error::new_spanned(segment, message);
    match &segment.arguments {
        PathArguments::AngleBracketed(argument) => {
            match unwrap_punctuated_first(&argument.args, error.clone())? {
                GenericArgument::Type(nested_type) => Ok(nested_type),
                _ => Err(error.clone()),
            }
        }
        _ => Err(error.clone()),
    }
}

pub fn get_nested_types<'a>(
    segment: &'a PathSegment,
    message: &'static str,
) -> Result<Vec<&'a Type>, Error> {
    let error = Error::new_spanned(segment, message);
    match &segment.arguments {
        PathArguments::AngleBracketed(arguments) => arguments
            .args
            .iter()
            .map(|argument| match argument {
                GenericArgument::Type(nested_type) => Ok(nested_type),
                _ => Err(error.clone()),
            })
            .collect(),
        _ => Err(error.clone()),
    }
}

#[inline]
pub fn unwrap_type_path<'a>(ty: &'a Type, message: &'static str) -> Result<&'a TypePath, Error> {
    match ty {
        Type::Path(type_path) => Ok(type_path),
        _ => Err(Error::new_spanned(ty, message)),
    }
}

#[inline]
pub fn get_lit_str<U: Display>(lit: &Lit, ident: &U) -> Result<String, Error> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.value()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} lit to be a string", ident),
        )),
    }
}

pub fn get_lit_as_string<U: Display>(lit: &Lit, ident: &U) -> Result<String, Error> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.value()),
        Lit::Int(lit_int) => Ok(lit_int.to_string()),
        Lit::Float(lit_float) => Ok(lit_float.to_string()),
        Lit::Bool(lit_bool) => Ok(lit_bool.value.to_string()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} lit to be a string/integer/float/boll", ident),
        )),
    }
}

#[inline]
pub fn get_lit_int<T: FromStr, U: Display>(lit: &Lit, ident: &U) -> Result<T, Error>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match lit {
        Lit::Int(lit_int) => Ok(lit_int.base10_parse().unwrap()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} lit to be a integer", ident),
        )),
    }
}

#[inline]
pub fn get_lit_float<T: FromStr, U: Display>(lit: &Lit, ident: &U) -> Result<T, Error>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match lit {
        Lit::Float(lit_float) => Ok(lit_float.base10_parse().unwrap()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} lit to be a float", ident),
        )),
    }
}

#[inline]
pub fn get_lit_bool<U: Display>(lit: &Lit, ident: &U) -> Result<bool, Error> {
    match lit {
        Lit::Bool(lit_bool) => Ok(lit_bool.value),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} lit to be a bool", ident),
        )),
    }
}
