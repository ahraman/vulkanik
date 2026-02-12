pub mod attr;
pub mod expand;
pub mod util;

use proc_macro2::TokenStream;
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, spanned::Spanned};

use crate::xml::{
    attr::{FieldAttr, StructAttr, parse_attrs},
    expand::{
        AttrFieldRef, FieldMode, ParsedField, ParsedFields, ParsedStruct, StructMode, StructVariant,
    },
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
    Ok(parse_struct_data(name, attrs, data)?.expand())
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
        for (index, field) in fields.enumerate() {
            let mut attr_name = None;
            let mut field_mode = FieldMode::Attr;

            for attr in parse_attrs::<FieldAttr>(&field.attrs)? {
                match attr {
                    FieldAttr::Rename(rename) => attr_name = Some(rename.value),
                    FieldAttr::Items(_) => field_mode = FieldMode::Items,
                    FieldAttr::Text(_) => field_mode = FieldMode::Text,
                    FieldAttr::Mixed(_) => field_mode = FieldMode::Mixed,
                    FieldAttr::Ignore(_) => field_mode = FieldMode::Default,
                }
            }

            let mut new_mode = None;
            match field_mode {
                FieldMode::Default => {}
                FieldMode::Attr => {
                    let attr_name = attr_name.unwrap_or_else(|| {
                        util::rust_type_to_xml(
                            field
                                .ident.as_ref()
                                .expect(r#"attribute fields on tuple structs need a name: use `#[xml(rename = "...")]`"#))
                    });

                    attr_fields.push(AttrFieldRef {
                        index,
                        attr_name: attr_name,
                    })
                }
                FieldMode::Items | FieldMode::Text | FieldMode::Mixed => {
                    new_mode = Some(StructMode::Branch {
                        container_field_index: index,
                    });
                }
            }

            if let Some(new_mode) = new_mode {
                if mode.is_some() {
                    return Err(syn::Error::new(
                        field.span(),
                        "cannot have multiple container fields at once in a struct",
                    ));
                } else {
                    mode = Some(new_mode);
                }
            }

            items.push(ParsedField {
                name: field.ident,
                ty: field.ty,

                index,
                mode: field_mode,
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
