mod xml;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Xml, attributes(xml))]
pub fn from_xml_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    xml::derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
