use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr, Token, Type, punctuated::Punctuated};

pub enum StructVariant {
    Unit,
    Tuple,
    Braced,
}

pub enum StructMode {
    Leaf,
    Branch { container_field_index: usize },
}

pub struct ParsedStruct {
    pub mode: StructMode,

    pub name: Ident,
    pub element: LitStr,

    pub fields: ParsedFields,
}

impl ParsedStruct {
    pub fn expand(self) -> TokenStream {
        self.emit_impl()
    }

    fn emit_impl(&self) -> TokenStream {
        let parse_fn = self.emit_parse_fn();

        let type_name = &self.name;
        let element_name = &self.element;
        quote! {
            impl #type_name {
                pub(crate) const ELEMENT: &'static str = #element_name;

                #parse_fn
            }
        }
    }

    fn emit_parse_fn(&self) -> TokenStream {
        let parse_fn_body = self.emit_parse_fn_body();

        quote! {
            pub fn parse_xml_element<R: std::io::Read>(
                    reader: &mut xml::EventReader<R>,
                    element: String,
                    attributes: std::collections::HashMap<String, String>,
                ) -> Result<Self, Error> {
                #parse_fn_body
            }
        }
    }

    fn emit_parse_fn_body(&self) -> TokenStream {
        let attr_parsing = self.emit_parse_fn_attr_parsing();
        let content_parsing = self.emit_parse_fn_content_parsing();
        let return_stmt = self.emit_parse_fn_return_stmt();

        let preamble = quote! {
            if element != Self::ELEMENT {
                return Err(Error::UnexpectedStart(element, Self::ELEMENT.to_string()));
            }
        };

        quote! {
            #preamble
            #attr_parsing
            #content_parsing
            #return_stmt
        }
    }

    fn emit_parse_fn_attr_parsing(&self) -> TokenStream {
        quote! {
            attributes.check_empty(&element, reader)?;
        }
    }

    fn emit_parse_fn_content_parsing(&self) -> TokenStream {
        let accum_init_stmt = self.emit_parse_fn_content_accum_init_stmt();
        let match_stmt = self.emit_parse_fn_match_stmt();
        quote! {
            #accum_init_stmt
            loop {
                #match_stmt
            }
        }
    }

    fn emit_parse_fn_content_accum_init_stmt(&self) -> TokenStream {
        match self.mode {
            StructMode::Leaf => quote! {},
            StructMode::Branch {
                container_field_index,
            } => {
                let container_field = &self.fields.items[container_field_index];
                let ident = container_field.local_name();

                quote! { let mut #ident = Default::default(); }
            }
        }
    }

    fn emit_parse_fn_match_stmt(&self) -> TokenStream {
        let mut start_element = None;
        let mut characters = None;
        match self.mode {
            StructMode::Leaf => {}
            StructMode::Branch {
                container_field_index,
            } => {
                let container_field = &self.fields.items[container_field_index];
                let container_name = container_field.local_name();
                match container_field.mode {
                    FieldMode::Items => start_element = Some(quote! {
                        
                    }),
                    FieldMode::Text => todo!(),
                    FieldMode::Mixed => todo!(),
                    _ => panic!(
                        "unexpected state for fields: container field detected but there is none"
                    ),
                }
            }
        }

        quote! {
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
    }

    fn emit_parse_fn_return_stmt(&self) -> TokenStream {
        let initializer_list = self.fields.items.iter().fold(
            Punctuated::<TokenStream, Token![,]>::new(),
            |mut init, field| {
                init.push_value(field.initializer());
                init
            },
        );

        let initializer = match self.fields.variant {
            StructVariant::Unit => quote! { Self },
            StructVariant::Tuple => quote! { Self(#initializer_list) },
            StructVariant::Braced => quote! { Self { #initializer_list } },
        };

        quote! {
            Ok(#initializer)
        }
    }
}

pub struct ParsedFields {
    pub variant: StructVariant,

    pub items: Vec<ParsedField>,
    pub attr_fields: Vec<AttrFieldRef>,
}

pub enum FieldMode {
    Default,
    Attr,
    Items,
    Text,
    Mixed,
}

pub struct ParsedField {
    pub name: Option<Ident>,
    pub ty: Type,

    pub index: usize,
    pub mode: FieldMode,
}

impl ParsedField {
    fn initializer(&self) -> TokenStream {
        match self.mode {
            FieldMode::Default => return quote! { Default::default() },
            _ => {
                let ident = self.local_name();
                quote! { #ident }
            }
        }
    }

    fn local_name(&self) -> Ident {
        let default_name = match self.mode {
            FieldMode::Attr | FieldMode::Default => &format!("__tmp{index}", index = self.index),
            FieldMode::Items | FieldMode::Text | FieldMode::Mixed => "content",
        };

        self.name
            .clone()
            .unwrap_or_else(|| Ident::new(default_name, Span::call_site()))
    }
}

pub struct AttrFieldRef {
    pub index: usize,
    pub attr_name: LitStr,
}
