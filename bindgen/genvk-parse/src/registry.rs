use std::{collections::HashMap, io::Read};

use xml::{EventReader, reader::XmlEvent};

use crate::{Error, FromAttr, IntoMap, MapExt};

#[derive(Debug)]
pub enum Api {
    Vulkan,
    VulkanSc,
    VulkanBase,
}

impl FromAttr for Api {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match value {
            Some(value) => Ok(match value.as_str() {
                "vulkan" => Self::Vulkan,
                "vulkansc" => Self::VulkanSc,
                "vulkanbase" => Self::VulkanBase,
                _ => return Err(Some(value)),
            }),
            None => Err(None),
        }
    }
}

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

        attributes.check_empty(&element, reader)?;

        let mut content = String::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
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

        Ok(Self(content))
    }
}

#[derive(Debug)]
pub enum Content {
    Characters(String),
    Name(String),
    Type(String),
}

impl Content {
    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        attributes.check_empty(&element, reader)?;

        let mut content = String::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    return if name.local_name == element {
                        Ok(match element.as_str() {
                            "name" => Self::Name(content),
                            "type" => Self::Type(content),
                            _ => return Err(Error::UnknownStart(element)),
                        })
                    } else {
                        Err(Error::UnexpectedEnd(name.local_name, element))
                    };
                }
                XmlEvent::Characters(text) => content += text.as_str(),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }
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

        let comment = attributes.remove_attr(&element, "comment")?;
        attributes.check_empty(&element, reader)?;

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
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
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
    Types(Types),
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
            Types::TAG => Self::Types(Types::parse_xml_element(reader, element, attributes)?),
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

        let comment = attributes.remove_attr(&element, "comment")?;
        attributes.check_empty(&element, reader)?;

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
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
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
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let name = attributes.remove_attr(&element, "name")?;
        let protect = attributes.remove_attr(&element, "protect")?;
        let comment = attributes.remove_attr(&element, "comment")?;
        attributes.check_empty(&element, reader)?;

        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
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

        let comment = attributes.remove_attr(&element, "comment")?;
        attributes.check_empty(&element, reader)?;

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
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
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
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let name = attributes.remove_attr(&element, "name")?;
        let author = attributes.remove_attr(&element, "author")?;
        let contact = attributes.remove_attr(&element, "contact")?;
        let comment = attributes.remove_attr(&element, "comment")?;
        attributes.check_empty(&element, reader)?;

        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
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

#[derive(Debug)]
pub struct Types {
    pub comment: Option<String>,

    pub items: Vec<TypesItem>,
}

impl Types {
    pub(crate) const TAG: &'static str = "types";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let comment = attributes.remove_attr(&element, "comment")?;
        attributes.check_empty(&element, reader)?;

        let mut items = Vec::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => items.push(TypesItem::parse_xml_element(
                    reader,
                    name.local_name,
                    attributes.into_map(),
                )?),
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
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
pub enum TypesItem {
    Comment(Comment),
    Type(Type),
}

impl TypesItem {
    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        Ok(match element.as_str() {
            Comment::TAG => Self::Comment(Comment::parse_xml_element(reader, element, attributes)?),
            Type::TAG => Self::Type(Type::parse_xml_element(reader, element, attributes)?),
            _ => return Err(Error::UnknownStart(element)),
        })
    }
}

#[derive(Debug)]
pub struct Type {
    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub comment: Option<String>,

    pub kind: TypeKind,
}

impl Type {
    pub(crate) const TAG: &'static str = "type";

    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        if element != Self::TAG {
            return Err(Error::UnexpectedStart(element, Self::TAG.to_string()));
        }

        let api = attributes.remove_attr(&element, "api")?;
        let requires = attributes.remove_attr(&element, "requires")?;
        let comment = attributes.remove_attr(&element, "comment")?;

        let kind = TypeKind::parse_xml_element(reader, element, attributes)?;

        Ok(Self {
            api,
            requires,
            comment,
            kind,
        })
    }
}

#[derive(Debug)]
pub enum TypeKind {
    External(ExternalType),
    Include(IncludeType),
    Define(DefineType),
}

impl TypeKind {
    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        Ok(
            match attributes.remove_attr::<Option<String>>(&element, "category")? {
                Some(category) => match category.as_str() {
                    "include" => {
                        Self::Include(IncludeType::parse_xml_element(reader, element, attributes)?)
                    }
                    "define" => {
                        Self::Define(DefineType::parse_xml_element(reader, element, attributes)?)
                    }
                    _ => {
                        return Err(Error::InvalidAttr(
                            element,
                            "category".to_string(),
                            category,
                        ));
                    }
                },
                None => Self::External(ExternalType::parse_xml_element(
                    reader, element, attributes,
                )?),
            },
        )
    }
}

#[derive(Debug)]
pub struct ExternalType {
    pub name: String,
}

impl ExternalType {
    fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        let name = attributes.remove_attr(&element, "name")?;
        attributes.check_empty(&element, reader)?;

        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
                    }
                }
                XmlEvent::Characters(text) => return Err(Error::Text(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self { name })
    }
}

#[derive(Debug)]
pub struct IncludeType {
    pub name: String,

    pub content: Option<String>,
}

impl IncludeType {
    fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        let name = attributes.remove_attr(&element, "name")?;
        attributes.check_empty(&element, reader)?;

        let mut content = None;
        loop {
            match reader.next()? {
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::UnknownStart(name.local_name));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
                    }
                }
                XmlEvent::Characters(text) => {
                    content = Some(content.unwrap_or_default() + text.as_str())
                }
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self { name, content })
    }
}

#[derive(Debug)]
pub struct DefineType {
    pub name: Option<String>,

    pub content: Vec<Content>,
}

impl DefineType {
    fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        mut attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        let name = attributes.remove_attr(&element, "name")?;
        attributes.check_empty(&element, reader)?;

        let mut content = Vec::new();
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    content.push(Content::parse_xml_element(
                        reader,
                        name.local_name,
                        attributes.into_map(),
                    )?);
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
                        break;
                    } else {
                        return Err(Error::UnexpectedEnd(name.local_name, element));
                    }
                }
                XmlEvent::Characters(text) => content.push(Content::Characters(text)),
                XmlEvent::EndDocument => return Err(Error::Eof),
                _ => {}
            }
        }

        Ok(Self { name, content })
    }
}
