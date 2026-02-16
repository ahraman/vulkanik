use syn::{
    Attribute, Ident, LitStr, Type,
    parse::{Parse, ParseStream},
};

use crate::util::{self, EqAttr};

const XML_HELPER_ATTR: &'static str = "xml";

pub fn parse_attrs<T: Parse>(attrs: &[Attribute]) -> syn::Result<Vec<T>> {
    util::parse_attrs(XML_HELPER_ATTR, attrs)
}

#[allow(dead_code)]
pub enum StructAttr {
    Rename(EqAttr<LitStr>),
    Incomplete(Ident),
    Text(Ident),
    Items(Ident, Type),
    Mixed(Ident, Type),
    Inline(Ident),
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "rename" => Self::Rename(EqAttr::parse(ident, input)?),
            "incomplete" => Self::Incomplete(ident),
            "text" => Self::Text(ident),
            "items" => Self::Items(ident, util::parse_parenthesized(input)?),
            "mixed" => Self::Mixed(ident, util::parse_parenthesized(input)?),
            "inline" => Self::Inline(ident),
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected struct attribute",
                ));
            }
        })
    }
}

#[allow(dead_code)]
pub enum FieldAttr {
    Rename(EqAttr<LitStr>),
    Text(Ident),
    Items(Ident),
    Content(Ident),
    Inner(Ident),
    Ignore(Ident),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "rename" => Self::Rename(EqAttr::parse(ident, input)?),
            "text" => Self::Text(ident),
            "items" => Self::Items(ident),
            "content" => Self::Content(ident),
            "inner" => Self::Inner(ident),
            "ignore" => Self::Ignore(ident),
            _ => {
                return Err(syn::Error::new_spanned(ident, "unexpected field attribute"));
            }
        })
    }
}

#[allow(dead_code)]
pub enum EnumAttr {
    Element(Ident),
    Attr(Ident, Option<LitStr>),
    Rename(EqAttr<LitStr>),
    Incomplete(Ident),
}

impl Parse for EnumAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "element" => Self::Element(ident),
            "attr" => Self::Attr(ident, util::parse_parenthesized(input).ok()),
            "rename" => Self::Rename(EqAttr::parse(ident, input)?),
            "incomplete" => Self::Incomplete(ident),
            _ => {
                return Err(syn::Error::new_spanned(ident, "unexpected enum attribute"));
            }
        })
    }
}

#[allow(dead_code)]
pub enum VariantAttr {
    Attr(EqAttr<LitStr>),
    Value(EqAttr<LitStr>),
    Default(Ident),
    None(Ident),
}

impl Parse for VariantAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "attr" => Self::Attr(EqAttr::parse(ident, input)?),
            "value" => Self::Value(EqAttr::parse(ident, input)?),
            "default" => Self::Default(ident),
            "none" => Self::None(ident),
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected variant attribute",
                ));
            }
        })
    }
}
