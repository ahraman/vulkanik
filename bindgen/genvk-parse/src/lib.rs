pub mod error;
pub mod registry;

use std::{collections::HashMap, io::Read};

use xml::{EventReader, attribute::OwnedAttribute, common::Position, reader::XmlEvent};

pub use crate::{
    error::{Error, TextError},
    registry::Registry,
};

pub fn parse_xml<R: Read>(reader: R) -> Result<Registry, TextError> {
    let mut reader = EventReader::new(reader);
    parse_xml_inner(&mut reader).map_err(|e| TextError::new(e, reader))
}

fn parse_xml_inner<R: Read>(reader: &mut EventReader<R>) -> Result<Registry, Error> {
    let mut registry = None;
    loop {
        match reader.next()? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                if registry
                    .replace(Registry::parse_xml_element(
                        reader,
                        name.local_name,
                        attributes.into_map(),
                    )?)
                    .is_some()
                {
                    return Err(Error::Duplicate(Registry::ELEMENT.to_string()));
                }
            }
            XmlEvent::EndDocument => return registry.ok_or_else(|| Error::Eof),
            XmlEvent::EndElement { name } => return Err(Error::UnknownEnd(name.local_name)),
            XmlEvent::Characters(text) => return Err(Error::Text(text)),
            _ => {}
        }
    }
}

trait IntoMap {
    fn into_map(self) -> HashMap<String, String>;
}

impl IntoMap for Vec<OwnedAttribute> {
    fn into_map(self) -> HashMap<String, String> {
        self.into_iter()
            .map(|attr| (attr.name.local_name, attr.value))
            .collect()
    }
}

trait MapExt {
    fn remove_attr<T: FromAttr>(&mut self, element: &str, attr: &str) -> Result<T, Error>;

    fn check_empty(&self, element: &str, pos: &impl Position) -> Result<(), Error>;
}

impl MapExt for HashMap<String, String> {
    fn remove_attr<T: FromAttr>(&mut self, element: &str, attr: &str) -> Result<T, Error> {
        T::from_attr(self.remove(attr)).map_err(|e| {
            e.map(|value| Error::InvalidAttr(element.to_string(), attr.to_string(), value))
                .unwrap_or_else(|| Error::MissingAttr(element.to_string(), attr.to_string()))
        })
    }

    fn check_empty(&self, element: &str, pos: &impl Position) -> Result<(), Error> {
        if !self.is_empty() {
            println!(
                "[warning]: tag `<{element}> at {} has unrecognized attributes",
                pos.position()
            );
        }

        Ok(())
    }
}

trait FromAttr: Sized {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>>;
}

impl FromAttr for String {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        value.ok_or(None)
    }
}

impl<T: FromAttr> FromAttr for Option<T> {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match T::from_attr(value) {
            Ok(value) => Ok(Some(value)),
            Err(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl<T: FromAttr> FromAttr for Vec<T> {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match value {
            Some(value) => value
                .split(',')
                .map(|s| T::from_attr(Some(s.to_string())))
                .collect(),
            None => Err(None),
        }
    }
}
