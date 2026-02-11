pub mod attr;
pub mod expand;
pub mod util;

use proc_macro2::TokenStream;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident};

use crate::xml::{
    attr::{StructAttr, parse_attrs},
    expand::{ParsedField, ParsedFields, ParsedStruct, StructMode, StructVariant},
};

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

fn derive_struct(name: Ident, attrs: Vec<Attribute>, data: DataStruct) -> syn::Result<TokenStream> {
    parse_struct_data(name, attrs, data)?.expand()
}

fn parse_struct_data(
    name: Ident,
    attrs: Vec<Attribute>,
    data: DataStruct,
) -> syn::Result<ParsedStruct> {
    let mut element = None;
    for attr in parse_attrs::<StructAttr>(&attrs)? {
        match attr {
            StructAttr::Rename(rename) => element = Some(rename.value),
        }
    }

    let element = element.unwrap_or_else(|| util::rust_type_to_xml(&name));
    let (mode, fields) = parse_struct_fields_data(data.fields)?;

    Ok(ParsedStruct {
        mode,

        name,
        element,

        fields,
    })
}

fn parse_struct_fields_data(fields: Fields) -> syn::Result<(StructMode, ParsedFields)> {
    let (variant, fields) = match fields {
        Fields::Unit => (StructVariant::Unit, None),
        Fields::Unnamed(fields) => (StructVariant::Tuple, Some(fields.unnamed.into_iter())),
        Fields::Named(fields) => (StructVariant::Braced, Some(fields.named.into_iter())),
    };

    let mut mode = None;
    let mut items = Vec::new();
    let mut attr_fields = Vec::new();

    if let Some(fields) = fields {
        for field in fields {
            items.push(ParsedField {
                name: field.ident,
                ty: field.ty,
            });
        }
    }

    let mode = mode.unwrap_or(StructMode::Leaf);
    Ok((
        mode,
        ParsedFields {
            variant,

            items,
            attr_fields,
        },
    ))
}

fn derive_enum(_data: DataEnum) -> syn::Result<TokenStream> {
    todo!()
}
