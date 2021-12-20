use std::convert::TryFrom;

use serde::Serialize;
use serde_json::value::RawValue;

use crate::{escape_json_parse, Options};

/// Prevent (hidden, not impossible) implementation of crate traits outside this crate.
pub trait Sealed {}

/// A [`Serialize`] value that has yet to be serialized.
pub struct NotYetSerialized<'a, T: Serialize>(pub &'a T);

impl<'a, T: Serialize> From<&'a T> for NotYetSerialized<'a, T> {
    fn from(input: &'a T) -> Self {
        Self(input)
    }
}

/// A [`Serialize`] value that has been serialized exactly once.
pub struct Serialized(Box<RawValue>);

impl<'a, T: Serialize> TryFrom<NotYetSerialized<'a, T>> for Serialized {
    type Error = serde_json::Error;

    fn try_from(value: NotYetSerialized<'_, T>) -> Result<Self, Self::Error> {
        serde_json::to_string(value.0)
            .and_then(RawValue::from_string)
            .map(Self)
    }
}

impl Serialized {
    /// Transform the serialized data into a valid JavaScript string.
    pub fn into_javascript_string_literal(self, options: &Options) -> String {
        escape_json_parse(&self.0, options)
    }
}
