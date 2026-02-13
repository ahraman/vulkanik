use syn::{
    Attribute, Ident, LitStr, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::xml::util::{self, parse_parenthesized};

const XML_HELPER_ATTR: &'static str = "xml";

pub fn parse_attrs<T: Parse>(attrs: &[Attribute]) -> syn::Result<Vec<T>> {
    let mut buf = Vec::new();
    for attr in attrs {
        if attr.path().is_ident(XML_HELPER_ATTR) {
            let iter = attr
                .parse_args_with(Punctuated::<T, Token![,]>::parse_terminated)?
                .into_iter();
            buf.extend(iter);
        }
    }

    Ok(buf)
}

#[allow(dead_code)]
pub struct EqAttr<T> {
    pub ident: Ident,
    pub value: T,
    pub eq_token: Token![=],
}

impl<T: Parse> EqAttr<T> {
    pub fn parse(ident: Ident, input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Err(syn::Error::new(
                input.span(),
                "unexpected end of input, expected `= {value}`",
            ))
        } else {
            let eq_token = input.parse::<Token![=]>()?;
            let value = input.parse()?;

            Ok(Self {
                ident,
                value,
                eq_token,
            })
        }
    }
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
}

impl Parse for EnumAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "element" => Self::Element(ident),
            "attr" => Self::Attr(ident, parse_parenthesized(input).ok()),
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
