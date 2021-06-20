use super::utils;

use proc_macro::TokenStream;
use quote::quote;

use syn::spanned::Spanned;
use syn::Error;
use syn::Result;

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream> {
    let span = input.span();
    let fields = utils::parse_fields(input.data, span, is_valid_field)?;
    let generics = input.generics;
    let struct_type = input.ident;

    let mut body = quote! {
        let len = span_info.x_len() * span_info.y_len() * span_info.z_len();
    };

    let mut constructor_fields = quote! {};

    for field in &fields {
        // convert the field identifier to a string literal
        // so `write_dataarray` understands it
        let lit = syn::LitStr::new(&field.to_string(), proc_macro2::Span::call_site());

        body = quote! {
            #body
            #[allow(unused_variables)]
            let (data, #field) = vtk::xml_parse::parse_dataarray(&data, #lit, len)?;
        };

        // do it in this order because the first item will be
        // `,field` if it were reversed (compiler error)
        // whereas this has a trailing comma (no error)
        //
        // for some reason #(#fields), does not produce the correct output below so
        // we manually concatenate here
        constructor_fields = quote! {
            #field, #constructor_fields
        };
    }

    // declare the whole trait
    let expanded = quote! {
        impl #generics vtk::traits::ParseDataArray for #struct_type #generics {
            fn parse_dataarrays(data:&str, span_info: &vtk::LocationSpans) -> Result<Self, vtk::NomErrorOwned> {
                #body

                Ok(
                    Self {
                        #constructor_fields
                    }
                )
            }
        }
    };

    // Hand the output tokens back to the compiler
    Ok(TokenStream::from(expanded))
}

fn is_valid_field(field_type: &syn::Type) -> Result<()> {
    match field_type {
        syn::Type::Path(path) => {
            // check that the overall path is Vec<Float>
            utils::inner_type_vec_float(&path.path, field_type.span())
        }
        //syn::Type::Slice(slice) => {
        //    // check that the T in &[T] is a float
        //    utils::inner_type_float(&slice.elem)
        //}
        //syn::Type::Reference(reference) => {
        //    // this is a reference to either a vector or a slice in order to be valid
        //    // so we just recurse backwards
        //    is_valid_field(&reference.elem)
        //}
        _ => {
            // unhandled type to export to dataarray
            Err(Error::new(
                field_type.span(),
                "unhandled datatype. Only accepts Vec<f64> and &[f64]",
            ))
        }
    }
}