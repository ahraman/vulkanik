#![allow(dead_code)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructMode {
    Leaf,
    Branch,
    Container,
}

pub struct ParsedStruct {
    pub mode: StructMode,

    pub name: Ident,
    pub element: LitStr,
}

impl ParsedStruct {
    pub fn generate(self) -> syn::Result<TokenStream> {
        self.generate_impl()
    }

    fn generate_impl(&self) -> syn::Result<TokenStream> {
        let parse_fn = self.generate_parse_fn()?;

        let type_name = &self.name;
        let element_name = &self.element;
        Ok(quote! {
            impl #type_name {
                pub(crate) const ELEMENT: &'static str = #element_name;

                #parse_fn
            }
        })
    }

    fn generate_parse_fn(&self) -> syn::Result<TokenStream> {
        let parse_fn_body = self.generate_parse_fn_body()?;

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

    fn generate_parse_fn_body(&self) -> syn::Result<TokenStream> {
        let preamble = self.generate_parse_fn_preamble()?;
        let attr_parsing = self.generate_parse_fn_attr_parsing()?;
        let content_parsing = self.generate_parse_fn_content_parsing()?;
        let return_stmt = self.generate_parse_fn_return_stmt()?;

        Ok(quote! {
            #preamble
            #attr_parsing
            #content_parsing
            #return_stmt
        })
    }

    fn generate_parse_fn_preamble(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            if element != Self::ELEMENT {
                return Err(Error::UnexpectedStart(element, Self::ELEMENT.to_string()));
            }
        })
    }

    fn generate_parse_fn_attr_parsing(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            attributes.check_empty(&element, reader)?;
        })
    }

    fn generate_parse_fn_content_parsing(&self) -> syn::Result<TokenStream> {
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

    fn generate_parse_fn_return_stmt(&self) -> syn::Result<TokenStream> {
        Ok(quote! {
            Ok(Self(content))
        })
    }
}
