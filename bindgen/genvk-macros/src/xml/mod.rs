mod parse;
pub mod util;

use proc_macro2::TokenStream;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Ident};

use crate::xml::parse::{ParsedStruct, StructMode};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    match input.data {
        Data::Struct(data) => derive_struct(input.ident, input.attrs, data),
        Data::Enum(data) => derive_enum(data),
        Data::Union(data) => Err(syn::Error::new_spanned(
            data.union_token,
            "union types unsupported",
        )),
    }
}

fn derive_struct(
    name: Ident,
    _attrs: Vec<Attribute>,
    _data: DataStruct,
) -> syn::Result<TokenStream> {
    let element = util::rust_type_to_xml(&name);

    let mode = StructMode::Leaf;

    let parsed = ParsedStruct {
        name,
        element,
        mode,
    };

    parsed.generate()
}

fn derive_enum(_data: DataEnum) -> syn::Result<TokenStream> {
    todo!()
}
