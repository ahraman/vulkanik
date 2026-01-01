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
                    return Err(Error::Duplicate(Registry::TAG.to_string()));
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
    fn remove_required(&mut self, element: &str, attr: &str) -> Result<String, Error>;

    fn check_empty(&self, element: &str, pos: &impl Position) -> Result<(), Error>;
}

impl MapExt for HashMap<String, String> {
    fn remove_required(&mut self, element: &str, attr: &str) -> Result<String, Error> {
        self.remove(attr)
            .ok_or_else(|| Error::MissingAttr(element.to_string(), attr.to_string()))
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
