use std::{collections::HashMap, io::Read};

use genvk_macros::Xml;
use xml::{EventReader, reader::XmlEvent};

use crate::{
    Error,
    traits::{FromAttr, IntoMap, MapExt, PushText},
};

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

#[derive(Debug, Xml)]
#[xml(text)]
pub struct Comment(#[xml(text)] pub String);

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

impl PushText for Vec<Content> {
    fn push_text(&mut self, text: String) {
        self.push(Content::Characters(text));
    }
}

#[derive(Debug, Xml)]
#[xml(items(RegistryItem))]
pub struct Registry {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<RegistryItem>,
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
            Comment::ELEMENT => {
                Self::Comment(Comment::parse_xml_element(reader, element, attributes)?)
            }
            Platforms::ELEMENT => {
                Self::Platforms(Platforms::parse_xml_element(reader, element, attributes)?)
            }
            Tags::ELEMENT => Self::Tags(Tags::parse_xml_element(reader, element, attributes)?),
            Types::ELEMENT => Self::Types(Types::parse_xml_element(reader, element, attributes)?),
            _ => return Err(Error::UnknownStart(element)),
        })
    }
}

#[derive(Debug, Xml)]
#[xml(items(Platform))]
pub struct Platforms {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<Platform>,
}

#[derive(Debug, Xml)]
pub struct Platform {
    pub name: String,
    pub protect: String,
    pub comment: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(items(Tag))]
pub struct Tags {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<Tag>,
}

#[derive(Debug, Xml)]
pub struct Tag {
    pub name: String,
    pub author: String,
    pub contact: String,
    pub comment: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(items(TypesItem))]
pub struct Types {
    pub comment: Option<String>,

    #[xml(items)]
    pub items: Vec<TypesItem>,
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
            Comment::ELEMENT => {
                Self::Comment(Comment::parse_xml_element(reader, element, attributes)?)
            }
            Type::ELEMENT => Self::Type(Type::parse_xml_element(reader, element, attributes)?),
            _ => return Err(Error::UnknownStart(element)),
        })
    }
}

#[derive(Debug, Xml)]
#[xml(inline)]
pub struct Type {
    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub comment: Option<String>,

    #[xml(inner)]
    pub kind: TypeKind,
}

#[derive(Debug)]
pub enum TypeKind {
    External(ExternalType),
    Include(IncludeType),
    Define(DefineType),
    Base(BaseType),
    Bitmask(BitmaskType),
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
                    "basetype" => {
                        Self::Base(BaseType::parse_xml_element(reader, element, attributes)?)
                    }
                    "bitmask" => {
                        Self::Bitmask(BitmaskType::parse_xml_element(reader, element, attributes)?)
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

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct ExternalType {
    pub name: String,
}

#[derive(Debug, Xml)]
#[xml(text, incomplete)]
pub struct IncludeType {
    pub name: String,

    #[xml(text)]
    pub content: Option<String>,
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct DefineType {
    pub name: Option<String>,

    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct BaseType {
    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug)]
pub enum BitmaskType {
    Decl(BitmaskTypeDecl),
    Alias(BitmaskTypeAlias),
}

impl BitmaskType {
    pub fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        Ok(if attributes.contains_key("alias") {
            Self::Alias(BitmaskTypeAlias::parse_xml_element(
                reader, element, attributes,
            )?)
        } else {
            Self::Decl(BitmaskTypeDecl::parse_xml_element(
                reader, element, attributes,
            )?)
        })
    }
}

#[derive(Debug, Xml)]
#[xml(mixed(Content), incomplete)]
pub struct BitmaskTypeDecl {
    #[xml(rename = "bitvalues")]
    pub bit_values: Option<String>,

    #[xml(content)]
    pub content: Vec<Content>,
}

#[derive(Debug, Xml)]
#[xml(incomplete)]
pub struct BitmaskTypeAlias {
    pub name: String,
    pub alias: String,
}
