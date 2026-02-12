pub mod attr;
pub mod expand;
pub mod util;

use proc_macro2::{Span, TokenStream};
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident};

use crate::xml::{
    attr::{FieldAttr, StructAttr, parse_attrs},
    expand::{
        AttrFieldRef, ParsedField, ParsedFields, ParsedStruct, StructContent, StructContentKind,
        StructVariant,
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
    let mut content_kind = None;
    let mut incomplete = false;
    for attr in parse_attrs::<StructAttr>(&attrs)? {
        match attr {
            StructAttr::Rename(rename) => element = Some(rename.value),
            StructAttr::Incomplete(_) => incomplete = true,
            StructAttr::Text(_) => content_kind = Some(StructContentKind::Text),
            StructAttr::Items(_, ty) => content_kind = Some(StructContentKind::Items(ty)),
            StructAttr::Mixed(_, ty) => content_kind = Some(StructContentKind::Mixed(ty)),
            StructAttr::Inline(_) => content_kind = Some(StructContentKind::Inline),
        }
    }

    let element = element.unwrap_or_else(|| util::rust_type_to_xml(&name));
    let (fields, content) = parse_struct_fields_data(data.fields, content_kind)?;

    Ok(ParsedStruct {
        name,
        element,

        fields,
        content,
        incomplete,
    })
}

fn parse_struct_fields_data(
    fields: Fields,
    content_kind: Option<StructContentKind>,
) -> syn::Result<(ParsedFields, Option<StructContent>)> {
    let (variant, fields) = match fields {
        Fields::Unit => (StructVariant::Unit, None),
        Fields::Unnamed(fields) => (StructVariant::Tuple, Some(fields.unnamed.into_iter())),
        Fields::Named(fields) => (StructVariant::Braced, Some(fields.named.into_iter())),
    };

    let mut items = Vec::new();
    let mut attr_fields = Vec::new();
    let mut content_field_index = usize::MAX;

    if let Some(fields) = fields {
        for (index, field) in fields.enumerate() {
            let mut attr_field = true;
            let mut default_initialized = false;
            let mut local_name_preference = None;
            let mut attr_name = None;

            for attr in parse_attrs::<FieldAttr>(&field.attrs)? {
                match (&content_kind, attr) {
                    (_, FieldAttr::Rename(rename)) => attr_name = Some(rename.value),
                    (_, FieldAttr::Ignore(_)) => {
                        attr_field = false;
                        default_initialized = true;
                    }
                    (Some(StructContentKind::Items(_)), FieldAttr::Items(_)) => {
                        attr_field = false;
                        local_name_preference = Some("items");
                    }
                    (Some(StructContentKind::Text), FieldAttr::Text(_))
                    | (Some(StructContentKind::Mixed(_)), FieldAttr::Content(_))
                    | (Some(StructContentKind::Inline), FieldAttr::Inner(_)) => {
                        attr_field = false;
                        local_name_preference = Some("content");
                    }
                    _ => {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "incompatible field with the structure content type",
                        ));
                    }
                }
            }

            if attr_field {
                let attr_name = attr_name.unwrap_or_else(|| {
                    util::rust_type_to_xml(
                        field
                            .ident
                            .as_ref()
                            .expect("name mandatory on attribute fields of a tuple struct"),
                    )
                });

                attr_fields.push(AttrFieldRef { index, attr_name });
            } else {
                content_field_index = index;
            }

            items.push(ParsedField {
                name: field.ident,
                ty: field.ty,

                index,
                default_initialized,
                local_name_preference,
            });
        }
    }

    if content_kind.is_some() && content_field_index == usize::MAX {
        return Err(syn::Error::new(
            Span::call_site(),
            "missing content field on branching struct",
        ));
    }

    Ok((
        ParsedFields {
            variant,

            items,
            attr_fields,
        },
        content_kind.map(|kind| StructContent {
            kind,
            field_index: content_field_index,
        }),
    ))
}

fn derive_enum(_data: DataEnum) -> syn::Result<TokenStream> {
    todo!()
}
