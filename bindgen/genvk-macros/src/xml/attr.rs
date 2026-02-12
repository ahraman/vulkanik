#![allow(dead_code)]

use syn::{
    Attribute, Ident, LitStr, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

use crate::xml::util;

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
            let eq_token = input.parse()?;
            let value = input.parse()?;

            Ok(Self {
                ident,
                value,
                eq_token,
            })
        }
    }
}

pub enum StructAttr {
    Rename(EqAttr<LitStr>),
    Text(Ident),
    Items(Ident, Type),
    Mixed(Ident, Type),
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "rename" => Self::Rename(EqAttr::parse(ident, input)?),
            "text" => Self::Text(ident),
            "items" => Self::Items(ident, util::parse_parenthesized(input)?),
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected struct attribute",
                ));
            }
        })
    }
}

pub enum FieldAttr {
    Rename(EqAttr<LitStr>),
    Text(Ident),
    Items(Ident),
    Mixed(Ident),
    Ignore(Ident),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "rename" => Self::Rename(EqAttr::parse(ident, input)?),
            "items" => Self::Items(ident),
            "text" => Self::Text(ident),
            "mixed" => Self::Mixed(ident),
            "ignore" => Self::Ignore(ident),
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected struct attribute",
                ));
            }
        })
    }
}
