use std::{collections::HashMap, io::Read};

use xml::{EventReader, common::Position, reader::XmlEvent};

use crate::{Error, IntoMap};

#[derive(Debug)]
pub struct Comment(pub String);

impl Comment {
    pub(crate) const TAG: &'static str = "comment";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        if !attributes.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                reader.position()
            );
        }

        let mut content = String::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, Self::TAG.to_string()));
                    }
                }
                XmlEvent::Characters(text) => content += text.as_str(),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self(content))
    }
}

#[derive(Debug)]
pub struct Registry {
    pub comment: Option<String>,

    pub items: Vec<RegistryItem>,
}

impl Registry {
    pub(crate) const TAG: &'static str = "registry";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let comment = attributes.remove("comment");

        if !attributes.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                reader.position()
            );
        }

        let mut items = Vec::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => items.push(RegistryItem::parse_xml_element(
                    reader,
                    name.local_name,
                    attributes.into_map(),
                )?),
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, Self::TAG.to_string()));
                    }
                }
                XmlEvent::Characters(text) => return Err(Error::Text(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self { comment, items })
    }
}

#[derive(Debug)]
pub enum RegistryItem {
    Comment(Comment),
    Platforms(Platforms),
    Tags(Tags),
}

impl RegistryItem {
    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        Ok(match element.as_str() {
            Comment::TAG => Self::Comment(Comment::parse_xml_element(reader, element, attributes)?),
            Platforms::TAG => {
                Self::Platforms(Platforms::parse_xml_element(reader, element, attributes)?)
            }
            Tags::TAG => Self::Tags(Tags::parse_xml_element(reader, element, attributes)?),
            _ => return Err(Error::UnknownStart(element)),
        })
    }
}

#[derive(Debug)]
pub struct Platforms {
    pub comment: Option<String>,

    pub items: Vec<Platform>,
}

impl Platforms {
    pub(crate) const TAG: &'static str = "platforms";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let comment = attributes.remove("comment");

        if !attributes.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                reader.position()
            );
        }

        let mut items = Vec::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => items.push(Platform::parse_xml_element(
                    reader,
                    name.local_name,
                    attributes.into_map(),
                )?),
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, Self::TAG.to_string()));
                    }
                }
                XmlEvent::Characters(text) => return Err(Error::Text(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self { comment, items })
    }
}

#[derive(Debug)]
pub struct Platform {
    pub name: String,
    pub protect: String,
    pub comment: Option<String>,
}

impl Platform {
    pub(crate) const TAG: &'static str = "platform";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(
                element.to_string(),
                Self::TAG.to_string(),
            ));
        }

        let name = attributes
            .remove("name")
            .ok_or_else(|| Error::MissingAttr(element.to_string(), "name".to_string()))?;
        let protect = attributes
            .remove("protect")
            .ok_or_else(|| Error::MissingAttr(element.to_string(), "protect".to_string()))?;
        let comment = attributes.remove("comment");

        if !attributes.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                reader.position()
            );
        }

        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, Self::TAG.to_string()));
                    }
                }
                XmlEvent::Characters(text) => return Err(Error::Text(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self {
            name,
            protect,
            comment,
        })
    }
}

#[derive(Debug)]
pub struct Tags {
    pub comment: Option<String>,

    pub items: Vec<Tag>,
}

impl Tags {
    pub(crate) const TAG: &'static str = "tags";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let comment = attributes.remove("comment");

        if !attributes.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                reader.position()
            );
        }

        let mut items = Vec::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => items.push(Tag::parse_xml_element(
                    reader,
                    name.local_name,
                    attributes.into_map(),
                )?),
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, Self::TAG.to_string()));
                    }
                }
                XmlEvent::Characters(text) => return Err(Error::Text(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self { comment, items })
    }
}

#[derive(Debug)]
pub struct Tag {
    pub name: String,
    pub author: String,
    pub contact: String,
    pub comment: Option<String>,
}

impl Tag {
    pub(crate) const TAG: &'static str = "tag";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(
                element.to_string(),
                Self::TAG.to_string(),
            ));
        }

        let name = attributes
            .remove("name")
            .ok_or_else(|| Error::MissingAttr(element.to_string(), "name".to_string()))?;
        let author = attributes
            .remove("author")
            .ok_or_else(|| Error::MissingAttr(element.to_string(), "author".to_string()))?;
        let contact = attributes
            .remove("contact")
            .ok_or_else(|| Error::MissingAttr(element.to_string(), "contact".to_string()))?;
        let comment = attributes.remove("comment");

        if !attributes.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                reader.position()
            );
        }

        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, Self::TAG.to_string()));
                    }
                }
                XmlEvent::Characters(text) => return Err(Error::Text(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self {
            name,
            author,
            contact,
            comment,
        })
    }
}
