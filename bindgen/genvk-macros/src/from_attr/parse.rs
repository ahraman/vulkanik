use syn::{DataEnum, Fields, Ident, LitStr, Variant, spanned::Spanned};

use super::attr::{VariantAttr, parse_attrs};

pub struct ParsedEnum {
    pub name: Ident,

    pub variants: Vec<ParsedVariant>,
}

pub struct ParsedVariant {
    pub name: Ident,
    pub name_str: LitStr,
}

pub fn parse_enum(name: Ident, data: DataEnum) -> syn::Result<ParsedEnum> {
    let variants = parse_enum_variants(data.variants)?;

    Ok(ParsedEnum { name, variants })
}

fn parse_enum_variants(
    variants: impl IntoIterator<Item = Variant>,
) -> syn::Result<Vec<ParsedVariant>> {
    let mut items = Vec::new();
    for variant in variants {
        let mut name_str = None;
        for attr in parse_attrs::<VariantAttr>(&variant.attrs)? {
            match attr {
                VariantAttr::Rename(rename) => name_str = Some(rename.value),
            }
        }

        match variant.fields {
            Fields::Unit => {}
            fields => {
                return Err(syn::Error::new(
                    fields.span(),
                    "complex enum variants unsupported; implement `FromAttr` manually.",
                ));
            }
        }

        let name = variant.ident;
        let name_str = name_str.unwrap_or_else(|| LitStr::new(&name.to_string(), name.span()));
        items.push(ParsedVariant { name, name_str });
    }

    Ok(items)
}
