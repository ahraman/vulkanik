pub mod error;
pub mod registry;
pub(crate) mod traits;

use std::io::Read;

use xml::{EventReader, reader::XmlEvent};

use crate::traits::{IntoMap};
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
