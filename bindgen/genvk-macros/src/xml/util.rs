use syn::{
    Ident, LitStr, Type, parenthesized,
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
