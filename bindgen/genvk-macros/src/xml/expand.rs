use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Type};

pub enum StructVariant {
    Unit,
    Tuple,
    Braced,
}

pub struct ParsedFields {
    pub variant: StructVariant,

    pub items: Vec<ParsedField>,
    pub attr_fields: Vec<AttrFieldRef>,
}

pub struct ParsedField {
    pub name: Option<Ident>,
    pub ty: Type,
}

pub struct AttrFieldRef {
    pub index: usize,
    pub attr_name: LitStr,
}

pub enum StructMode {
    Leaf,
    Branch { items_field_index: usize },
    Container { content_field_index: usize },
}

pub struct ParsedStruct {
    pub mode: StructMode,

    pub name: Ident,
    pub element: LitStr,

    pub fields: ParsedFields,
}

impl ParsedStruct {
    pub fn expand(self) -> syn::Result<TokenStream> {
        self.emit_impl()
    }

    fn emit_impl(&self) -> syn::Result<TokenStream> {
        let parse_fn = self.emit_parse_fn()?;

        let type_name = &self.name;
        let element_name = &self.element;
        Ok(quote! {
            impl #type_name {
                pub(crate) const ELEMENT: &'static str = #element_name;

                #parse_fn
            }
        })
    }

    fn emit_parse_fn(&self) -> syn::Result<TokenStream> {
        let parse_fn_body = self.emit_parse_fn_body()?;

        Ok(quote! {
            pub fn parse_xml_element<R: std::io::Read>(
                    reader: &mut xml::EventReader<R>,
                    element: String,
                    attributes: std::collections::HashMap<String, String>,
                ) -> Result<Self, Error> {
                #parse_fn_body
            }
        })
    }

    fn emit_parse_fn_body(&self) -> syn::Result<TokenStream> {
        let preamble = self.emit_parse_fn_preamble()?;
        let attr_parsing = self.emit_parse_fn_attr_parsing()?;
        let content_parsing = self.emit_parse_fn_content_parsing()?;
        let return_stmt = self.emit_parse_fn_return_stmt()?;

        Ok(quote! {
            #preamble
            #attr_parsing
            #content_parsing
            #return_stmt
        })
    }

    fn emit_parse_fn_preamble(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            if element != Self::ELEMENT {
                return Err(Error::UnexpectedStart(element, Self::ELEMENT.to_string()));
            }
        })
    }

    fn emit_parse_fn_attr_parsing(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            attributes.check_empty(&element, reader)?;
        })
    }

    fn emit_parse_fn_content_parsing(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            let mut content = String::new();
            loop {
                match reader.next()? {
                    XmlEvent::StartElement { name, .. } => {
                        return Err(Error::UnknownStart(name.local_name));
                    }
                    XmlEvent::EndElement { name } => {
                        if name.local_name == Self::ELEMENT {
                            break;
                        } else {
                            return Err(Error::UnexpectedEnd(name.local_name, element));
                        }
                    }
                    XmlEvent::Characters(text) => content += text.as_str(),
                    XmlEvent::EndDocument => return Err(Error::Eof),
                    _ => {}
                }
            }
        })
    }

    fn emit_parse_fn_return_stmt(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            Ok(Self(content))
        })
    }
}
