use std::collections::HashMap;

use xml::{attribute::OwnedAttribute, common::Position};

use crate::Error;

pub(super) trait FromAttr: Sized {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>>;
}

impl FromAttr for bool {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match value {
            None => Err(None),
            Some(value) => match value.as_str().parse() {
                Ok(value) => Ok(value),
                Err(_) => Err(Some(value)),
            },
        }
    }
}

impl FromAttr for i32 {
    fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
        match value {
            None => Err(None),
            Some(value) => match value.as_str().parse() {
                Ok(value) => Ok(value),
                Err(_) => Err(Some(value)),
            },
        }
    }
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

pub(super) trait IntoMap {
    fn into_map(self) -> HashMap<String, String>;
}

impl IntoMap for Vec<OwnedAttribute> {
    fn into_map(self) -> HashMap<String, String> {
        self.into_iter()
            .map(|attr| (attr.name.local_name, attr.value))
            .collect()
    }
}

pub(super) trait MapExt {
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

pub(super) trait PushText: Sized {
    fn push_text(&mut self, text: String);
}

impl PushText for String {
    fn push_text(&mut self, text: String) {
        *self += &text;
    }
}

impl PushText for Option<String> {
    fn push_text(&mut self, text: String) {
        *self.get_or_insert(String::new()) += &text;
    }
}
