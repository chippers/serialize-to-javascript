extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_derive(TemplateData)]
pub fn derive_template_data(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::DeriveInput);
    let item_span = item.span();
    let name = item.ident;
    match item.data {
        syn::Data::Struct(data) => {
            let (impl_generics, ty_generics, _) = item.generics.split_for_impl();
            let mut replacements = TokenStream2::new();
            for field in data.fields {
                match field.ident {
                    Some(ident) => {
                        let template_ident = quote::format_ident!("__TEMPLATE_{}__", ident);
                        replacements.append_all(quote!(
                            let template: String = template.into();
                            let data: Serialized = NotYetSerialized(&self.#ident).try_into()?;
                            let template = template.replace(
                                stringify!(#template_ident),
                                &data.into_javascript_string_literal()
                            );
                        ));
                    }
                    None => {
                        return syn::Error::new(
                            field.span(),
                            "TemplateData fields must all have names",
                        )
                        .to_compile_error()
                        .into()
                    }
                }
            }
            quote!(
                impl #impl_generics ::serialize_to_javascript::private::Sealed for #name #ty_generics {}
                impl #impl_generics ::serialize_to_javascript::TemplateData for #name #ty_generics {
                    fn render(&self, template: &str) -> ::serialize_to_javascript::Result<String> {
                        use ::serialize_to_javascript::private::NotYetSerialized;
                        use ::serialize_to_javascript::private::Serialized;

                        #replacements

                        Ok(template)
                    }
                }
            )
        }
        _ => {
            return syn::Error::new(
                item_span,
                "TemplateData currently only supports data structs",
            )
            .to_compile_error()
            .into()
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn default_template(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as syn::LitStr);
    let item = parse_macro_input!(item as syn::DeriveInput);
    let name = item.ident;
    let (impl_generics, ty_generics, _) = item.generics.split_for_impl();
    quote!(
        #item
        impl #impl_generics ::serialize_to_javascript::DefaultTemplate for #name #ty_generics {
            const RAW_TEMPLATE: &'static str = include_str!(#path);
        }
    )
    .into()
}
