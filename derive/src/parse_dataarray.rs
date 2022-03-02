use super::utils;

use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::TokenStream as TokenStream2;

use syn::spanned::Spanned;
use syn::Result;
use crate::dataarray::Encoding;

use darling::{ast, FromDeriveInput, FromField, FromMeta};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(vtk), supports(struct_any))]
struct InputReceiver{
    /// The struct ident.
    ident: syn::Ident,

    /// The type's generics. 
    generics: syn::Generics,

    // only work on structs
    data: ast::Data<(), FieldReceiver>,

    #[darling(default)]
    encoding: Encoding,
}

#[derive(Debug, FromField)]
#[darling(attributes(vtk))]
struct FieldReceiver {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    #[allow(dead_code)]
    ty: syn::Type,
}

#[derive(Debug)]
struct ValidatedField {
    ident: syn::Ident,
    ty: syn::Type
}

struct Visitor {
    name: syn::Ident,
    tokens: proc_macro2::TokenStream
}

fn create_visitor(original_struct: &syn::Ident, fields: &[ValidatedField]) -> Visitor {
    // first find out what we are naming the struct
    let mut visitor_name = original_struct.to_string();
    visitor_name.push_str("Visitor");
    let ident = syn::Ident::new(&visitor_name, original_struct.span());

    
    let trait_impl = create_visitor_trait_impl(&ident, original_struct, fields);
    let struct_def = create_visitor_struct_definition(&ident, fields);
    let tokens = quote!(
        #struct_def

        #trait_impl
    );
    Visitor { tokens, name: ident }
}

fn create_visitor_struct_definition(visitor_name: &syn::Ident, fields: &[ValidatedField]) -> proc_macro2::TokenStream {
    let mut out = quote!();

    for field in fields {
        let field_name = &field.ident;

        out = quote!(
            #out
            #field_name: vtk::parse::PartialDataArrayBuffered,
        );
    }

    quote!(
        pub struct #visitor_name {
            #out
        }
    )
}

fn create_visitor_trait_impl(visitor_name: &syn::Ident, original_name: &syn::Ident, fields: &[ValidatedField]) -> proc_macro2::TokenStream {
    let span_type = quote!(vtk::mesh::Spans3D);

    let read_headers = visitor_read_headers(visitor_name, fields);
    let append_to_buffer = visitor_buffer_append(fields);
    let finish = visitor_finish(original_name, fields);

 
    let out = quote!(
        impl vtk::Visitor<#span_type> for #visitor_name {
            type Output = #original_name;

            fn read_headers<'a>(spans: &#span_type, buffer: &'a [u8]) -> nom::IResult<&'a [u8], Self> {
                #read_headers
            }

            fn add_to_appended_reader<'a, 'b>(
                &'a self,
                buffer: &'b mut Vec<std::cell::RefMut<'a, parse::OffsetBuffer>>,
            ) {
                #append_to_buffer
            }

            fn finish(self, spans: &#span_type) -> Result<Self::Output, vtk::ParseError> {
                #finish
            }
        }
    );

    out
}

/// builds the body of `Visitor::read_headers`
fn visitor_read_headers(visitor_name: &syn::Ident, fields: &[ValidatedField]) -> proc_macro2::TokenStream {
    let mut out = quote!(
        let rest = buffer;
    );

    for field in fields {
        println!(":::::: HERE");

        let fieldname = &field.ident;
        let lit = syn::LitByteStr::new(&fieldname.to_string().as_bytes(), fieldname.span());
        dbg!(&lit);

        // TODO: fix this size estimation somehow?
        out = quote!(
            #out
            let (rest, #fieldname) = vtk::parse::parse_dataarray_or_lazy(rest, #lit, 0)?;
            let #fieldname = parse::PartialDataArrayBuffered::new(#fieldname, 0);
        );
    }

    //
    // build the comma separated fields
    //
    let comma_fields = make_fields_comma_separated(fields);

    out = quote!(
        #out

        let visitor = #visitor_name {
            #comma_fields
        };

        Ok((rest, visitor))
    );


    out
}

/// places all the fields in a comma separated list
fn make_fields_comma_separated(fields: &[ValidatedField]) -> proc_macro2::TokenStream {
    
    let mut out= quote!();

    for field in fields {
        let fieldname = &field.ident;

        // TODO: fix this size estimation somehow?
        out = quote!(
            #out
            #fieldname,
        );
    }

    out
}

/// builds the body of `Visitor::add_to_appended_reader`
fn visitor_buffer_append(fields: &[ValidatedField]) -> proc_macro2::TokenStream {
    let mut out = quote!();

    for field in fields {
        let fieldname = &field.ident;
        out = quote!(
            #out
            self.#fieldname.append_to_reader_list(buffer);
        );
    }

    out
}

/// builds the body of `Visitor::finish`
fn visitor_finish(output_ident: &syn::Ident, fields: &[ValidatedField]) -> proc_macro2::TokenStream {
    let mut out = quote!();

    for field in fields {
        let fieldname = &field.ident;

        out = quote!(
            #out
            let comp  = self.#fieldname.components();
            let #fieldname = self.#fieldname.into_buffer();
            let #fieldname = vtk::FromBuffer::from_buffer(#fieldname, &spans, comp);
        )
    }

    let comma_sep_fields = make_fields_comma_separated(fields);

    quote!(
        #out 
        Ok(#output_ident { #comma_sep_fields} )
    )
}

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream> {

    let receiver = InputReceiver::from_derive_input(&input).unwrap();

    let InputReceiver {
        ref ident,
        ref generics,
        data,
        ..
    } = receiver;

    let (imp, ty, wher) = generics.split_for_impl();

    let fields : Result<Vec<_>> = data
        .take_struct()
        .expect("Should never be enum")
        .fields
        .into_iter()
        .map(|field: FieldReceiver| {
            if let Some(ident) = &field.ident {
                Ok(ValidatedField { ident: ident.clone(), ty: field.ty })
            } else {
                Err(syn::Error::new(field.ty.span(), "does not handle tuple struct"))
            }
            
        })
        .collect();
    let fields = fields?;


    let Visitor { name: visitor_name, tokens: visitor_tokens}  = create_visitor(&ident, &fields);

    let out = quote!(
        #visitor_tokens

        impl #imp vtk::ParseArray for #ident #ty #wher {
            type Visitor = #visitor_name;
        }
    );

    Ok(out.into())
}
