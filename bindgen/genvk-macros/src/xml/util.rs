use syn::{Ident, LitStr};

pub fn rust_type_to_xml(ident: &Ident) -> LitStr {
    let name = ident.to_string().to_lowercase();
    LitStr::new(&name, ident.span())
}
