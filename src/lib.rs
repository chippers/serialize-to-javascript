//! Serialize [`serde::Serialize`] values to JavaScript.
//!
//! Templated names that are replaced inside templates are `__TEMPLATE_my_field__` where `my_field`
//! is a field on a struct implementing [`Template`]. Raw value template names use
//! `__RAW_my_field__`.
//!
//! ```rust
//! use serialize_to_javascript::{Template, default_template};
//! #[derive(Template)]
//! struct Keygen<'a> {
//!     key: &'a str,
//!     length: usize,
//!
//!     #[raw]
//!     optional_script: &'static str
//! }
//! ```

use serde_json::value::RawValue;
pub use serde_json::{Error, Result};

pub use serialize_to_javascript_impl::{default_template, Template};

#[doc(hidden)]
pub mod private;

/// Optional setting to pass to the templating system.
#[derive(Debug, Default, Copy, Clone)]
pub struct Options {
    /// Should parsed objects be deep frozen with [`Object.freeze()`]?
    ///
    /// This flag currently does nothing.
    ///
    /// [`Object.freeze()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/freeze
    #[allow(dead_code)]
    pub freeze: bool,

    /// Extra amount of bytes to allocate while serializing JSON to a JavaScript string.
    ///
    /// Note: this is not the total buffer size, but the extra buffer size created. By default the
    /// buffer size will be the same as the serialized field value if it needs no extra escaping.
    pub buf: usize,
}

/// A struct that contains [`serde::Serialize`] data to insert into a template.
///
/// Create this automatically with a `#[derive(Template)]` attribute. All fields not marked `#[raw]`
/// will be compile-time checked that they implement [`serde::Serialize`].
///
/// Due to the nature of templating variables, [tuple structs] are not allowed as their fields
/// have no names. [Unit structs] have no fields and are a valid target of this trait.
///
/// Template variables are generated as `__TEMPLATE_my_field__` where the serialized value of the
/// `my_field` field replaces all instances of the template variable.
///
/// # Raw Values
///
/// If you have raw values you would like to inject into the template that is not serializable
/// through JSON, such as a string of JavaScript code, then you can mark a field with `#[raw]` to
/// make it embedded directly. **Absolutely NO serialization occurs**, the field is just turned into
/// a string using [`Display`]. As such, fields that are marked `#[raw]` _only_ require [`Display`].
///
/// Raw values use `__RAW_my_field__` as the template variable.
///
/// ---
///
/// This trait is sealed.
///
/// [tuple structs]: https://doc.rust-lang.org/book/ch05-01-defining-structs.html#using-tuple-structs-without-named-fields-to-create-different-types
/// [`Display`]: std::fmt::Display
pub trait Template: self::private::Sealed {
    /// Render the serialized template data into the passed template.
    fn render(&self, template: &str, options: &Options) -> Result<String>;
}

/// A [`Template`] with an attached default template.
///
/// Create this automatically with `#[default_template("myfile.js")` on your [`Template`] struct.
pub trait DefaultTemplate: Template {
    /// The raw static string with the templates contents.
    ///
    /// When using `#[default_template("myfile.js")]` it will be generated as
    /// `include_str!("myfile.js")`.
    const RAW_TEMPLATE: &'static str;

    /// Render the serialized template data into the default template.
    ///
    /// If this method is implemented manually, it still needs to use [`Template::render`] to be
    /// serialized correctly.
    fn render_default(&self, options: &Options) -> Result<String> {
        self.render(Self::RAW_TEMPLATE, options)
    }
}

/// Transforms & escapes a JSON String to `JSON.parse('{json}')`
///
/// Single quotes chosen because double quotes are already used in JSON. With single quotes, we only
/// need to escape strings that include backslashes or single quotes. If we used double quotes, then
/// there would be no cases that a string doesn't need escaping.
///
/// # Safety
///
/// The ability to safely escape JSON into a JSON.parse('{json}') relies entirely on 2 things.
///
/// 1. `serde_json`'s ability to correctly escape and format json into a [`String`].
/// 2. JavaScript engines not accepting anything except another unescaped, literal single quote
///     character to end a string that was opened with it.
///
/// # Allocations
///
/// A new [`String`] will always be allocated. If `buf_size` is set to `0`, then it will by default
/// allocate to the size of the [`RawValue`] +  14 characters to cover the `JSON.parse('')` wrapper.
pub fn escape_json_parse(json: &RawValue, options: &Options) -> String {
    let json = json.get();

    // dynamically set the buffer size
    let buf = {
        // 14 chars in JSON.parse('')
        let mut buf = 14;

        // we know it's at least going to contain the length of the json
        buf += json.len();

        // add in user defined extra buffer size
        buf += options.buf;

        // freezing code expands the output size due to the embedded reviver code
        if options.freeze {
            // todo: set this to the length of the JSON.parse() reviver code
            buf += 0;
        }

        buf
    };

    let mut s = String::with_capacity(buf);
    s.push_str("JSON.parse('");

    // insert a backslash before any backslash or single quote characters to escape them
    let mut last = 0;
    for (idx, _) in json.match_indices(|c| c == '\\' || c == '\'') {
        s.push_str(&json[last..idx]);
        s.push('\\');
        last = idx;
    }

    // finish appending the trailing json characters that don't need escaping
    s.push_str(&json[last..]);

    // close out the escaped JavaScript string
    s.push('\'');

    // custom reviver to freeze all parsed items
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/parse#using_the_reviver_parameter
    if options.freeze {
        // todo: write the freezing reviver
    }

    // finish the JSON.parse() call
    s.push(')');

    s
}
