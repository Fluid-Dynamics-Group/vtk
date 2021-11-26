use syn::Error;
use syn::Result;

use proc_macro2::Span;

pub(crate) fn parse_fields(
    input_data: syn::Data,
    span: Span,
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
