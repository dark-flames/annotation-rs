use syn::punctuated::Punctuated;
use syn::{
    Error as SynError, Error, GenericArgument, Lit, MetaList, NestedMeta, Path, PathArguments,
    PathSegment, Type, TypePath,
};

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
pub fn unwrap_punctuated_last<T, P>(
    punctuated: &Punctuated<T, P>,
    error: Error,
) -> Result<&T, Error> {
    match punctuated.last() {
        Some(s) => Ok(s),
        None => Err(error),
    }
}

#[inline]
pub fn get_nested_type(segment: &PathSegment, message: &str) -> Result<&Type, Error> {
    let error = Error::new_spanned(segment, message);
    match &segment.arguments {
        PathArguments::AngleBracketed(argument) => {
            match unwrap_punctuated_first(&argument.args, error)? {
                GenericArgument::Type(nested_type) => Ok(nested_type),
                _ => Err(error),
            }
        }
        _ => Err(error),
    }
}

#[inline]
pub fn unwrap_type_path(ty: &Type, message: &str) -> Result<&TypePath, Error> {
    match ty {
        Type::Path(type_path) => Ok(type_path),
        _ => Err(Error::new_spanned(ty, message)),
    }
}

#[inline]
pub fn get_lit_str(lit: &Lit, path: &Path) -> Result<String, Error> {
    match lit {
        Lit::Str(lit_str) => Ok(lit_str.value()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a string", path),
        )),
    }
}

#[inline]
pub fn get_lit_int<T>(lit: &Lit, path: &Path) -> Result<T, Error> {
    match lit {
        Lit::Int(lit_int) => Ok(lit_int.base10_parse::<T>()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a integer", path),
        )),
    }
}

#[inline]
pub fn get_lit_int_str(lit: &Lit, path: &Path) -> Result<String, Error> {
    match lit {
        Lit::Int(lit_int) => Ok(lit_int.to_string()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a integer", path),
        )),
    }
}

#[inline]
pub fn get_lit_float<T>(lit: &Lit, path: &Path) -> Result<T, Error> {
    match lit {
        Lit::Float(lit_float) => Ok(lit_float.base10_parse::<T>()()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a float", path),
        )),
    }
}

#[inline]
pub fn get_lit_float_str(lit: &Lit, path: &Path) -> Result<String, Error> {
    match lit {
        Lit::Float(lit_float) => Ok(lit_float.to_string()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a float", path),
        )),
    }
}

#[inline]
pub fn get_lit_bool(lit: &Lit, path: &Path) -> Result<bool, Error> {
    match lit {
        Lit::Bool(lit_bool) => Ok(lit_bool.value()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a bool", path),
        )),
    }
}

#[inline]
pub fn get_lit_bool_str(lit: &Lit, path: &Path) -> Result<String, Error> {
    match lit {
        Lit::Bool(lit_bool) => Ok(lit_bool.to_string()),
        _ => Err(Error::new_spanned(
            lit,
            format!("expected {} attribute to be a bool", path),
        )),
    }
}
