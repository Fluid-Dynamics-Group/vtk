use super::utils;

use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::TokenStream as TokenStream2;

use syn::spanned::Spanned;
use syn::Result;

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream> {
    let span = input.span();
    let fields = utils::parse_fields(input.data, span)?;
    let generics = input.generics;
    let struct_type = input.ident;

    let mut body = quote! {
        let len = span_info.x_len() * span_info.y_len() * span_info.z_len();
        let mut offset_buffers = Vec::<&mut vtk::parse::OffsetBuffer>::new();
    };

    // the output fields that will be used to generate the constructor,
    // in the form of 
    // {field_1, field_2, field_3}
    let mut constructor_fields = quote! ();

    for field in &fields {
        // convert the field identifier to a string literal
        // so `write_dataarray` understands it
        let lit_bytes = syn::LitByteStr::new(&field.to_string().as_bytes(), proc_macro2::Span::call_site());

        body = quote! {
            #body
            #[allow(unused_variables)]
            let (data, #field) = vtk::parse::parse_dataarray_or_lazy(&data, #lit_bytes, len)?;
            let mut #field = vtk::parse::PartialDataArrayBuffered::new(#field, len);
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

    // unpack the location values into so that we can also add them to the data arrays
    body = quote!(
        #body 
        let mut locations_x__ = vtk::parse::PartialDataArrayBuffered::new(location_partial.x, len);
        let mut locations_y__ = vtk::parse::PartialDataArrayBuffered::new(location_partial.y, len);
        let mut locations_z__ = vtk::parse::PartialDataArrayBuffered::new(location_partial.z, len);
    );


    // match on the value for a buffer, add it to `offset_buffers` if we still
    // need to parse it from the appended section
    fn add_to_array(body: TokenStream2, ident_name: TokenStream2) -> TokenStream2 {
        quote!(
            #body

            match &mut #ident_name {
                vtk::parse::PartialDataArrayBuffered::AppendedBinary(offset) => {
                    offset_buffers.push(offset)
                }
                _ => ()
            }
        )
    }

    // add all of the values to the `offset_buffers` that was previously declared
    // so that we can iterate through them and mutate them
    body = add_to_array(body, quote!(locations_x__));
    body = add_to_array(body, quote!(locations_y__));
    body = add_to_array(body, quote!(locations_z__));

    // add any of the unparsed fields from the struct fields to `offset_buffers` as well
    for field in &fields {
        body = add_to_array(body, quote!(#field));
    }

    // now we need to go through all the fields and mutate them in place 
    body = quote!(
        #body 

        // if we have any binary data:
        if offset_buffers.len() > 0 {
            //we have some data to read - first organize all of the data by the offsets
            offset_buffers.sort_unstable();

            let mut iterator = offset_buffers.iter_mut().peekable();
            let (mut appended_data, _) = vtk::parse::setup_appended_read(data)?;

            loop {
                if let Some(current_offset_buffer) = iterator.next() {
                    // get the number of bytes to read based on the next element's offset
                    let reading_offset = iterator.peek()
                        .map(|offset_buffer|  vtk::parse::AppendedArrayLength::Known((offset_buffer.offset - current_offset_buffer.offset) as usize))
                        .unwrap_or(vtk::parse::AppendedArrayLength::UntilEnd);

                    let (remaining_appended_data, _) = vtk::parse::parse_appended_binary(appended_data, reading_offset, &mut current_offset_buffer.buffer)?;
                    appended_data = remaining_appended_data
                } else {
                    // there are not more elements in the array - lets leave
                    break
                }
            }
        }
    );

    //
    // now we unpack all the locations since they have definitely been parsed now
    //

    body = quote!(
        #body 

        let locations = vtk::Locations {
            x_locations: locations_x__.into_buffer(), 
            y_locations: locations_y__.into_buffer(), 
            z_locations: locations_z__.into_buffer(), 
        };
    );

    // unpack each of the individual fields

    for field in &fields {
        body = quote!(
            #body
            let #field = #field.into_buffer();
        );
    }

    // declare the whole trait
    let expanded = quote! {
        impl #generics vtk::traits::ParseDataArray for #struct_type #generics {
            fn parse_dataarrays(data:&[u8], span_info: &vtk::LocationSpans, location_partial: vtk::parse::LocationsPartial) -> Result<(Self, vtk::Locations), vtk::ParseError> {
                #body

                Ok(
                    (
                    Self {
                        #constructor_fields
                    },
                    locations
                    )
                )
            }
        }
    };

    // Hand the output tokens back to the compiler
    Ok(TokenStream::from(expanded))
}
