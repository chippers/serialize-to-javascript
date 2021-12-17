use serde::Serialize;
use serde_json::value::RawValue;
use std::convert::TryFrom;

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
    pub fn into_javascript_string_literal(self) -> String {
        escape_json_parse(&self.0)
    }
}

/// Transforms & escapes a JSON String -> JSON.parse('{json}')
///
/// Single quotes chosen because double quotes are already used in JSON. With single quotes, we only
/// need to escape strings that include backslashes or single quotes. If we used double quotes, then
/// there would be no cases that a string doesn't need escaping.
///
/// # Safety
///
/// The ability to safely escape JSON into a JSON.parse('{json}') relies entirely on 2 things.
///
/// 1. `serde_json`'s ability to correctly escape and format json into a string.
/// 2. JavaScript engines not accepting anything except another unescaped, literal single quote
///     character to end a string that was opened with it.
fn escape_json_parse(json: &RawValue) -> String {
    let json = json.get();

    // 14 chars in JSON.parse('')
    // todo: should we increase the 14 by x to allow x amount of escapes before another allocation?
    let mut s = String::with_capacity(json.len() + 14);
    s.push_str("JSON.parse('");

    // insert a backslash before any backslash or single quote characters.
    let mut last = 0;
    for (idx, _) in json.match_indices(|c| c == '\\' || c == '\'') {
        s.push_str(&json[last..idx]);
        s.push('\\');
        last = idx;
    }

    // finish appending the trailing characters that don't need escaping
    s.push_str(&json[last..]);
    s.push_str("')");
    s
}
