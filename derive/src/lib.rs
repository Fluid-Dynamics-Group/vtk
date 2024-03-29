mod dataarray;
mod parse_dataarray;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DataArray, attributes(vtk_write))]
pub fn derive_dataarray(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    dataarray::derive(input)
        .map(Into::into)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(ParseArray, attributes(vtk_parse))]
pub fn derive_parse_dataarray(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    parse_dataarray::derive(input)
        .map(Into::into)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
