use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitStr, Token, Type, punctuated::Punctuated};

use crate::xml::util;

pub enum StructVariant {
    Unit,
    Tuple,
    Braced,
}

pub enum StructContentKind {
    Text,
    Items(Type),
    Mixed(Type),
    Inline,
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
    pub incomplete: bool,
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
        let consts = self.emit_impl_consts();
        let parse_fn = self.emit_parse_fn();

        let type_name = &self.name;
        quote! {
            impl #type_name {
                #consts

                #parse_fn
            }
        }
    }

    fn emit_impl_consts(&self) -> TokenStream {
        if self.incomplete {
            TokenStream::new()
        } else {
            let element_name = &self.element;

            quote! {
                pub(crate) const ELEMENT: &'static str = #element_name;
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
        let preamble = self.emit_parse_fn_preamble();
        let attr_parsing = self.emit_parse_fn_attr_parsing();
        let content_parsing = self.emit_parse_fn_content_parsing();
        let return_stmt = self.emit_parse_fn_return_stmt();

        quote! {
            #preamble

            #attr_parsing

            #content_parsing

            #return_stmt
        }
    }

    fn emit_parse_fn_preamble(&self) -> TokenStream {
        if self.incomplete {
            TokenStream::new()
        } else {
            quote! {
                if element != Self::ELEMENT {
                    return Err(Error::UnexpectedStart(element, Self::ELEMENT.to_string()));
                }
            }
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

        let empty_check = if matches!(&self.content, Some(StructContent {kind: StructContentKind::Inline, ..})) {
            TokenStream::new()
        } else {
            quote!{ attributes.check_empty(&element, reader)?; }
        };

        quote! {
            #initializers

            #empty_check
        }
    }

    fn emit_parse_fn_content_parsing(&self) -> TokenStream {
        match self.content {
            Some(StructContent {
                kind: StructContentKind::Inline,
                field_index,
            }) => {
                let content_field = &self.fields.items[field_index];
                let content_field_name = content_field.local_name();
                let content_field_type = &content_field.ty;

                quote! {
                    let #content_field_name = #content_field_type::parse_xml_element(
                        reader,
                        element,
                        attributes,
                    )?;
                }
            }
            _ => {
                let accum_init_stmt = self.emit_parse_fn_content_accum_init_stmt();
                let match_stmt = self.emit_parse_fn_match_stmt();
                quote! {
                    #accum_init_stmt
                    loop {
                        #match_stmt
                    }
                }
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
                    StructContentKind::Text => {
                        if util::is_option_type(&content_field.ty) {
                            quote! { None }
                        } else {
                            quote! { String::new() }
                        }
                    }
                    StructContentKind::Items(_) | StructContentKind::Mixed(_) => {
                        quote! { Vec::new() }
                    }
                    StructContentKind::Inline => {
                        // inline structs delegate content initialization to their member field.
                        unreachable!()
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
                            XmlEvent::Characters(text) => { #content_field_name.push_text(text); },
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
                            XmlEvent::StartElement { name, attributes, .. } => {
                                #content_field_name.push(#ty::parse_xml_element(
                                    reader,
                                    name.local_name,
                                    attributes.into_map(),
                                )?);
                            }
                        });
                        characters = Some(quote! {
                            XmlEvent::Characters(text) => #content_field_name.push_text(text),
                        });
                    }
                    StructContentKind::Inline => {}
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
                    if name.local_name == element {
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
