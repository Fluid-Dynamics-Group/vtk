use proc_macro::TokenStream;
use quote::quote;
use syn::Result;

use darling::{ast, FromDeriveInput, FromField, FromMeta};

#[derive(FromMeta, Debug, Clone, Copy)]
enum Encoding {
    Ascii,
    Base64,
    Binary,
}

impl Default for Encoding {
    fn default() -> Self {
        Encoding::Binary
    }
}

/// Support parsing from a full derive input. Unlike FromMeta, this isn't
/// composable; each darling-dependent crate should have its own struct to handle
/// when its trait is derived.
#[derive(Debug, FromDeriveInput)]
// This line says that we want to process all attributes declared with `my_trait`,
// and that darling should panic if this receiver is given an enum.
#[darling(attributes(vtk), supports(struct_any))]
struct MyInputReceiver {
    /// The struct ident.
    ident: syn::Ident,

    /// The type's generics. You'll need these any time your trait is expected
    /// to work with types that declare generics.
    generics: syn::Generics,

    /// Receives the body of the struct or enum. We don't care about
    /// struct fields because we previously told darling we only accept structs.
    data: ast::Data<(), MyFieldReceiver>,

    /// The Input Receiver demands a volume, so use `Volume::Normal` if the
    /// caller doesn't provide one.
    #[darling(default)]
    encoding: Encoding,
}

#[derive(Debug, FromField)]
#[darling(attributes(vtk))]
struct MyFieldReceiver {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    #[allow(dead_code)]
    ty: syn::Type,
}

fn appended_encoding_body(fields: Vec<&MyFieldReceiver>) -> Result<proc_macro2::TokenStream> {

    let inline_arrays = quote!(Ok(()));
    let is_appended = quote!(true);
    let mut headers_body = quote!();
    let mut appended_body = quote!();

    for field in &fields {
        // convert the field identifier to a string literal
        // so `write_dataarray` understands it
        let field_name = &field.ident.as_ref().unwrap();
        let lit = syn::LitStr::new(&field_name.to_string(), proc_macro2::Span::call_site());

        headers_body = quote! {
            #headers_body

            let ref_field = &self.#field_name;
            let comps = vtk::Array::components(ref_field);

            vtk::write_appended_dataarray_header(writer, #lit, offset, comps)?;
            offset += (std::mem::size_of::<f64>() * self.#field_name.len()) as i64;
        }
    }

    for field in &fields {
        // convert the field identifier to a string literal
        // so `write_dataarray` understands it
        let field_name = &field.ident.as_ref().unwrap();

        appended_body = quote! {
            #appended_body

            vtk::Array::write_binary(&self.#field_name, writer)?;
        }
    }

    headers_body = quote!(
        #headers_body
        Ok(())
    );

    appended_body = quote!(
        #appended_body
        Ok(())
    );

    Ok(assemble_trait(
        inline_arrays,
        is_appended,
        headers_body,
        appended_body,
    ))
}

fn inline_encoding(fields: Vec<&MyFieldReceiver>, encoding: Encoding) -> Result<proc_macro2::TokenStream> {
    let mut inline_arrays = quote!();
    let is_appended = quote!(false);
    let headers_body = quote!(Ok(()));
    let appended_body = quote!(Ok(()));

    let vtk_encoding = match encoding {
        Encoding::Ascii => quote!(vtk::Encoding::Ascii),
        Encoding::Base64=> quote!(vtk::Encoding::Base64),
        _ => unreachable!()
    };

    for field in &fields {
        // convert the field identifier to a string literal
        // so `write_dataarray` understands it
        let field_name = &field.ident.as_ref().unwrap();
        let lit = syn::LitStr::new(&field_name.to_string(), proc_macro2::Span::call_site());

        inline_arrays = quote! {
            #inline_arrays

            vtk::write_inline_dataarray(writer, &self.#field_name, #lit, #vtk_encoding)?;
        }
    }

    inline_arrays = quote!(
        #inline_arrays
        Ok(())
    );

    Ok(assemble_trait(
        inline_arrays,
        is_appended,
        headers_body,
        appended_body,
    ))
}

fn assemble_trait(
    inline_arrays: proc_macro2::TokenStream,
    is_appended: proc_macro2::TokenStream,
    appended_headers: proc_macro2::TokenStream,
    appended_arrays: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote!(
        fn write_inline_dataarrays<W: std::io::Write>(
            &self,
            #[allow(unused_variables)] writer: &mut vtk::EventWriter<W>,
        ) -> Result<(), vtk::Error> {
            #inline_arrays
        }
        fn is_appended_array() -> bool {
            #is_appended
        }
        fn write_appended_dataarray_headers<W: std::io::Write>(
            &self,
            #[allow(unused_variables)]
            writer: &mut vtk::EventWriter<W>,
            #[allow(unused_variables)]
            mut offset: i64,
        ) -> Result<(), vtk::Error> {
            #appended_headers
        }
        fn write_appended_dataarrays<W: std::io::Write>(
            &self,
            #[allow(unused_variables)]
            writer: &mut vtk::EventWriter<W>,
        ) -> Result<(), vtk::Error> {
            #appended_arrays
        }
    )
}

pub fn derive(input: syn::DeriveInput) -> Result<TokenStream> {
    let receiver = MyInputReceiver::from_derive_input(&input).unwrap();

    let MyInputReceiver {
        ref ident,
        ref generics,
        ref data,
        encoding,
    } = receiver;

    let (imp, ty, wher) = generics.split_for_impl();
    let fields = data
        .as_ref()
        .take_struct()
        .expect("Should never be enum")
        .fields;

    let trait_body = 
        match encoding {
            Encoding::Ascii | Encoding::Base64 => inline_encoding(fields, encoding)?,
            Encoding::Binary => appended_encoding_body(fields)?
        };

    let out = quote! {
        impl #imp vtk::traits::DataArray for #ident #ty #wher {
            #trait_body
        }
    };

    Ok(out.into())
}
