use syn::spanned::Spanned;
use syn::Error;
use syn::Result;

use proc_macro2::Span;

pub(crate) fn parse_fields(
    input_data: syn::Data,
    span: Span,
    validate_type: impl Fn(&syn::Type) -> Result<()>,
) -> Result<Vec<syn::Ident>> {
    match input_data {
        syn::Data::Struct(data) => {
            match data.fields {
                syn::Fields::Named(fields) => {
                    let mut out_fields = vec![];

                    // get all of the names of the named fields
                    //
                    // we dont use iterators here so that we can early escape with `?`
                    for field in fields.named {
                        // safe to unwrap here since the struct have named fields
                        let field_name = field.ident.unwrap();

                        // make sure the datatype is correct
                        validate_type(&field.ty)?;

                        out_fields.push(field_name)
                    }

                    Ok(out_fields)
                }
                syn::Fields::Unnamed(_) => {
                    // error: dont accept unnnamed fields
                    Err(Error::new(
                        span,
                        "cannot derive for structs with unnnamed fields",
                    ))
                }
                syn::Fields::Unit => {
                    // dont accept unit structs
                    Err(Error::new(
                        span,
                        "cannot derive for structs with unnnamed fields",
                    ))
                }
            }
        }
        syn::Data::Enum(_) => Err(Error::new(span, "can only derive for structs")),
        syn::Data::Union(_) => Err(Error::new(span, "can only derive for structs")),
    }
}

/// Check that a given inner type T is a f32 / f64
pub(crate) fn inner_type_float(inner_field: &syn::Type) -> Result<()> {
    match inner_field {
        syn::Type::Path(typepath) => {
            // TODO: check that there is only 1 item here
            typepath
                .path
                .segments
                .first()
                .map(|x: &syn::PathSegment| {
                    x.ident == syn::Ident::new("f64", proc_macro2::Span::call_site())
                        || x.ident == syn::Ident::new("f32", proc_macro2::Span::call_site())
                })
                .ok_or(Error::new(inner_field.span(), "type is not f64"))?;
            Ok(())
        }
        _ => {
            // unknown datatype
            Err(Error::new(inner_field.span(), "type is not f64"))
        }
    }
}

/// Check that an inner type T is Vec<f32> or Vec<f64>
pub(crate) fn inner_type_vec_float(inner_field: &syn::Path, span: Span) -> Result<()> {
    return Ok(());

    if inner_field.segments.len() != 1 {
        return Err(Error::new(
            span,
            "more than one path segment. Expected Vec<f64> or &[f64]",
        ));
    }


    let mut fields_iter = inner_field.segments.iter();

    let first_field = fields_iter.next();

    // if the container is not a vector we cannot derive this
    let is_vec = first_field
        .map(|segment: &syn::PathSegment| segment.ident == syn::Ident::new("Vec", span))
        .ok_or(Error::new(span, "Missing path segment datatype"))?;

    if !is_vec {
        return Err(Error::new(
            span,
            "datatype was not a slice or vector of f64",
        ));
    }

    // we know that we have a value here, we can safely unwrap
    let first_field = first_field.unwrap();

    // check that the type arguments are floats
    if let syn::PathArguments::AngleBracketed(angle_args) = &first_field.arguments {
        if angle_args.args.len() != 1 {
            // we expected only a single float value here
            return Err(Error::new(
                first_field.arguments.span(),
                "more than 1 argument to Vec, this should not happen",
            ));
        }
        let arg = angle_args.args.first().unwrap();
        match arg {
            syn::GenericArgument::Type(ty) => inner_type_float(ty),
            _ => {
                // unkown inner type
                Err(Error::new(
                    first_field.arguments.span(),
                    "unknown inner type. Expected a f64",
                ))
            }
        }
    } else {
        Err(Error::new(
            first_field.arguments.span(),
            "Arguments did not used angled brackets. This should not happen for Vec<f64>",
        ))
    }
}
