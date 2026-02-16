use syn::{
    Attribute, Ident, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub fn is_option_type(ty: &Type) -> bool {
    match ty {
        Type::Path(ty) => ty
            .path
            .segments
            .first()
            .is_some_and(|ty| ty.ident.to_string().starts_with("Option")),
        _ => false,
    }
}

pub fn parse_parenthesized<T: Parse>(input: ParseStream) -> syn::Result<T> {
    let content;
    parenthesized!(content in input);
    content.parse()
}

pub fn parse_attrs<T: Parse>(helper: &'static str, attrs: &[Attribute]) -> syn::Result<Vec<T>> {
    let mut buf = Vec::new();
    for attr in attrs {
        if attr.path().is_ident(helper) {
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
