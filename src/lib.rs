#[doc(hidden)]
pub mod private;

pub use serde_json::{Error, Result};
pub use serialize_to_javascript_impl::{default_template, TemplateData};

/// This trait is sealed, do not attempt to implement it yourself.
pub trait TemplateData: self::private::Sealed {
    fn render(&self, template: &str) -> Result<String>;
}

pub trait DefaultTemplate: TemplateData {
    const RAW_TEMPLATE: &'static str;

    fn render_default(&self) -> Result<String> {
        self.render(Self::RAW_TEMPLATE)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
