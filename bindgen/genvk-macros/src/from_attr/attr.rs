use syn::{
    Attribute, Ident, LitStr,
    parse::{Parse, ParseStream},
};

use crate::util::{self, EqAttr};

const XML_HELPER_ATTR: &'static str = "attr";

pub fn parse_attrs<T: Parse>(attrs: &[Attribute]) -> syn::Result<Vec<T>> {
    util::parse_attrs(XML_HELPER_ATTR, attrs)
}

pub enum VariantAttr {
    Rename(EqAttr<LitStr>),
}

impl Parse for VariantAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        Ok(match name.as_str() {
            "rename" => Self::Rename(EqAttr::parse(ident, input)?),
            _ => {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unexpected variant attribute",
                ));
            }
        })
    }
}
