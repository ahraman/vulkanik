use std::{collections::HashMap, io::Read};

use xml::EventReader;

use crate::Error;

pub struct Registry {}

impl Registry {
    pub(crate) const TAG: &'static str = "registry";

    pub(crate) fn parse_xml_element<R: Read>(
        reader: &mut EventReader<R>,
        element: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self, Error> {
        todo!()
    }
}
