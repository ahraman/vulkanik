pub mod attr;
pub mod expand;
pub mod parse;

use proc_macro2::{Span, TokenStream};
use syn::{Data, DataEnum, DeriveInput, Ident};

use self::parse::parse_enum;

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    match input.data {
        Data::Enum(data) => derive_enum(input.ident, data),
        _ => Err(syn::Error::new(
            Span::call_site(),
            "`#[derive(FromAttr)]` unsupported on types other than enums",
        )),
    }
}

fn derive_enum(name: Ident, data: DataEnum) -> syn::Result<TokenStream> {
    Ok(parse_enum(name, data)?.expand())
}
