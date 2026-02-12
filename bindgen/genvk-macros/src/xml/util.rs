use syn::{
    Ident, LitStr, parenthesized,
    parse::{Parse, ParseStream},
};

pub fn rust_type_to_xml(ident: &Ident) -> LitStr {
    let name = ident.to_string().to_lowercase();
    LitStr::new(&name, ident.span())
}

pub fn parse_parenthesized<T: Parse>(input: ParseStream) -> syn::Result<T> {
    let content;
    parenthesized!(content in input);
    content.parse()
}
