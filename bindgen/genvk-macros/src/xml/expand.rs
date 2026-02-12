use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr, Token, Type, punctuated::Punctuated};

pub enum StructVariant {
    Unit,
    Tuple,
    Braced,
}

pub enum StructContentKind {
    Text,
    Items(Type),
    Mixed(Type),
}

pub struct StructContent {
    pub kind: StructContentKind,
    pub field_index: usize,
}

pub struct ParsedStruct {
    pub name: Ident,
    pub element: LitStr,

    pub fields: ParsedFields,
    pub content: Option<StructContent>,
}

pub struct ParsedFields {
    pub variant: StructVariant,

    pub items: Vec<ParsedField>,
    pub attr_fields: Vec<AttrFieldRef>,
}

#[allow(dead_code)]
pub struct ParsedField {
    pub name: Option<Ident>,
    pub ty: Type,

    pub index: usize,
    pub default_initialized: bool,
    pub local_name_preference: Option<&'static str>,
}

impl ParsedField {
    fn initializer(&self) -> TokenStream {
        if self.default_initialized {
            quote! { Default::default() }
        } else {
            let ident = self.local_name();
            quote! { #ident }
        }
    }

    fn local_name(&self) -> Ident {
        let default_name = format!("field{}", self.index);
        self.name.clone().unwrap_or_else(|| {
            Ident::new(
                self.local_name_preference.unwrap_or(&default_name),
                Span::call_site(),
            )
        })
    }
}

pub struct AttrFieldRef {
    pub index: usize,
    pub attr_name: LitStr,
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
                    mut attributes: std::collections::HashMap<String, String>,
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
        let mut initializers = TokenStream::new();
        for field_ref in &self.fields.attr_fields {
            let field = &self.fields.items[field_ref.index];
            let field_name = field.local_name();
            let attr_name = &field_ref.attr_name;
            initializers = quote! {
                #initializers
                let #field_name = attributes.remove_attr(&element, #attr_name)?;
            };
        }

        quote! {
            #initializers

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
        match &self.content {
            None => TokenStream::new(),
            Some(content) => {
                let content_field = &self.fields.items[content.field_index];
                let ident = content_field.local_name();

                let init_expr = match &content.kind {
                    StructContentKind::Text => quote! { String::new() },
                    StructContentKind::Items(_) | StructContentKind::Mixed(_) => {
                        quote! { Vec::new() }
                    }
                };
                quote! { let mut #ident = #init_expr; }
            }
        }
    }

    fn emit_parse_fn_match_stmt(&self) -> TokenStream {
        let mut start_element = None;
        let mut characters = None;
        match &self.content {
            None => {}
            Some(content) => {
                let content_field = &self.fields.items[content.field_index];
                let content_field_name = content_field.local_name();
                match &content.kind {
                    StructContentKind::Text => {
                        characters = Some(quote! {
                            XmlEvent::Characters(text) => #content_field_name += text.as_str(),
                        });
                    }
                    StructContentKind::Items(ty) => {
                        start_element = Some(quote! {
                                XmlEvent::StartElement { name, attributes, .. } => {
                                    #content_field_name.push(#ty::parse_xml_element(
                                        reader,
                                        name.local_name,
                                        attributes.into_map(),
                                )?);
                            }
                        });
                    }
                    StructContentKind::Mixed(ty) => {
                        start_element = Some(quote! {
                                XmlEvent::StartElement { name, .. } => {
                                    #content_field_name.push(#ty::parse_xml_element(
                                        reader,
                                        name.local_name,
                                        attributes.into_map(),
                                )?),
                            }
                        });
                        characters = Some(quote! {
                            XmlEvent::Characters(text) => #content_field_name.parse_text(text),
                        });
                    }
                }
            }
        }

        let start_element = start_element.unwrap_or_else(|| {
            quote! {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                },
            }
        });

        let characters = characters.unwrap_or_else(TokenStream::new);

        quote! {
            match reader.next()? {
                #start_element
                #characters
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::ELEMENT {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
                    }
                }
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }
    }

    fn emit_parse_fn_return_stmt(&self) -> TokenStream {
        let initializer_list = self.fields.items.iter().fold(
            Punctuated::<TokenStream, Token![,]>::new(),
            |mut init, field| {
                init.push(field.initializer());
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
