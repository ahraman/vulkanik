pub mod attr;
pub mod expand;
pub mod util;

use proc_macro2::{Span, TokenStream};
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, LitStr, Type, Variant,
    spanned::Spanned,
};

use crate::xml::{
    attr::{EnumAttr, FieldAttr, StructAttr, VariantAttr, parse_attrs},
    expand::{
        AttrFieldRef, ParsedEnum, ParsedField, ParsedFields, ParsedStruct, ParsedVariant,
        ParsedVariants, StructContent, StructContentKind, StructVariant,
    },
};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    match input.data {
        Data::Struct(data) => derive_struct(input.ident, input.attrs, data),
        Data::Enum(data) => derive_enum(input.ident, input.attrs, data),
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
    let (fields, content) = parse_struct_fields(data.fields, content_kind)?;

    Ok(ParsedStruct {
        name,
        element,

        fields,
        content,
        incomplete,
    })
}

fn parse_struct_fields(
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

fn derive_enum(name: Ident, attrs: Vec<Attribute>, data: DataEnum) -> syn::Result<TokenStream> {
    Ok(parse_enum(name, attrs, data)?.expand())
}

enum EnumKind {
    Element,
    Attr,
    Value(LitStr),
}

fn parse_enum(name: Ident, attrs: Vec<Attribute>, data: DataEnum) -> syn::Result<ParsedEnum> {
    let mut kind = EnumKind::Element;
    for attr in parse_attrs::<EnumAttr>(&attrs)? {
        kind = match attr {
            EnumAttr::Element(_) => EnumKind::Element,
            EnumAttr::Attr(_, attr_name) => match attr_name {
                None => EnumKind::Attr,
                Some(attr_name) => EnumKind::Value(attr_name),
            },
        }
    }

    let variants = parse_enum_variants(data.variants, kind)?;

    Ok(ParsedEnum { name, variants })
}

fn parse_enum_variants(
    variants: impl IntoIterator<Item = Variant>,
    kind: EnumKind,
) -> syn::Result<ParsedVariants> {
    match kind {
        EnumKind::Element => {
            let mut items = Vec::new();
            for variant in variants.into_iter() {
                for _ in parse_attrs::<VariantAttr>(&variant.attrs)? {
                    return Err(syn::Error::new(
                        variant.span(),
                        "attribute not supported on this enum kind",
                    ));
                }

                let ty = parse_enum_variant_type(variant.fields)?;
                items.push(ParsedVariant {
                    name: variant.ident,
                    ty,
                })
            }

            Ok(ParsedVariants::Elements { items })
        }
        EnumKind::Attr => {
            let mut default = None;
            let mut items = Vec::new();
            for variant in variants.into_iter() {
                let span = variant.span();

                let mut attr_name = None;
                let mut is_default = false;
                for attr in parse_attrs::<VariantAttr>(&variant.attrs)? {
                    match attr {
                        VariantAttr::Attr(attr) => attr_name = Some(attr.value),
                        VariantAttr::Default(_) => is_default = true,
                        VariantAttr::Value(_) | VariantAttr::None(_) => {
                            return Err(syn::Error::new(
                                span,
                                "attribute not supported on this enum kind",
                            ));
                        }
                    }
                }

                let ty = parse_enum_variant_type(variant.fields)?;
                let parsed_variant = ParsedVariant {
                    name: variant.ident,
                    ty,
                };

                if is_default {
                    if default.replace(parsed_variant).is_some() {
                        return Err(syn::Error::new(
                            span,
                            "cannot have multiple default variants",
                        ));
                    }
                } else {
                    let attr_name = attr_name.ok_or_else(|| {
                        syn::Error::new(
                            span,
                            "all variants need their attribute name specified, or declared as default",
                        )
                    })?;
                    items.push((parsed_variant, attr_name))
                }
            }

            let default = default.ok_or_else(|| {
                syn::Error::new(
                    Span::call_site(),
                    "default variant mandatory for this enum kind",
                )
            })?;

            Ok(ParsedVariants::Attrs { items, default })
        }
        EnumKind::Value(attr_name) => {
            let mut none = None;
            let mut default = None;
            let mut items = Vec::new();
            for variant in variants.into_iter() {
                let span = variant.span();

                let mut attr_value = None;
                let mut is_none = false;
                let mut is_default = false;
                for attr in parse_attrs::<VariantAttr>(&variant.attrs)? {
                    match attr {
                        VariantAttr::Value(value) => attr_value = Some(value.value),
                        VariantAttr::None(_) => is_none = true,
                        VariantAttr::Default(_) => is_default = true,
                        VariantAttr::Attr(_) => {
                            return Err(syn::Error::new(
                                span,
                                "attribute not supported on this enum kind",
                            ));
                        }
                    }
                }

                let ty = parse_enum_variant_type(variant.fields)?;
                let parsed_variant = ParsedVariant {
                    name: variant.ident,
                    ty,
                };

                if is_none && is_default {
                    return Err(syn::Error::new(
                        span,
                        "variant cannot have both `none` and `default` attributes",
                    ));
                } else if is_none {
                    if none.replace(parsed_variant).is_some() {
                        return Err(syn::Error::new(span, "cannot have multiple none variants"));
                    }
                } else if is_default {
                    if default.replace(parsed_variant).is_some() {
                        return Err(syn::Error::new(
                            span,
                            "cannot have multiple default variants",
                        ));
                    }
                } else {
                    let attr_value = attr_value.ok_or_else(|| {
                        syn::Error::new(
                        span,
                            "all variants need their attribute name specified, or declared as default",
                        )
                    })?;
                    items.push((parsed_variant, attr_value))
                }
            }

            Ok(ParsedVariants::AttrValues {
                attr_name,
                items,
                none,
                default,
            })
        }
    }
}

fn parse_enum_variant_type(fields: Fields) -> syn::Result<Type> {
    let span = fields.span();
    match fields {
        Fields::Unnamed(fields) => {
            let mut iter = fields.unnamed.into_iter();
            let field = iter.next();
            Ok(match (field, iter.next()) {
                (None, _) | (Some(_), Some(_)) => {
                    return Err(syn::Error::new(
                        span,
                        "variants must have exactly one unnamed field",
                    ));
                }
                (Some(field), None) => field.ty,
            })
        }
        _ => return Err(syn::Error::new(span, "named or unit variants unsupported")),
    }
}
