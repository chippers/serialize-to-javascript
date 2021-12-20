extern crate proc_macro;

use proc_macro::TokenStream;

use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, spanned::Spanned};

/// Checks if the passed type implements the passed trait.
fn trait_check(type_: syn::Type, trait_: TokenStream2) -> TokenStream2 {
    quote!(
        const _: fn() = || {
            fn assert_impl_all<T: ?Sized + #trait_>() {}
            assert_impl_all::<#type_>();
        };
    )
}

#[proc_macro_derive(Template, attributes(raw))]
pub fn derive_template(item: TokenStream) -> TokenStream {
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
                        let templated_field_name = format!("__TEMPLATE_{}__", ident);

                        // we expect self, template, and options bindings to exist
                        let data = if field.attrs.iter().any(|attr| attr.path.is_ident("raw")) {
                            let trait_check = trait_check(field.ty, quote!(::serde::Serialized));
                            quote!(
                                #trait_check

                                use ::std::convert::TryInto;
                                use ::serialize_to_javascript::private::{NotYetSerialized, Serialized};

                                let data: Serialized = NotYetSerialized(&self.#ident).try_into()?;
                                let data: String = data.into_javascript_string_literal(options);
                            )
                        } else {
                            let trait_check = trait_check(field.ty, quote!(::std::fmt::Display));
                            quote!(
                                #trait_check
                                let data: String = self.#ident.to_string();
                            )
                        };

                        replacements.append_all(quote!(
                            let template = template.replace(
                                #templated_field_name,
                                &{#data}
                            );
                        ));
                    }
                    None => {
                        return syn::Error::new(
                            field.span(),
                            "Template fields must all have names",
                        )
                            .to_compile_error()
                            .into();
                    }
                }
            }
            quote!(
                impl #impl_generics ::serialize_to_javascript::private::Sealed for #name #ty_generics {}
                impl #impl_generics ::serialize_to_javascript::Template for #name #ty_generics {
                    fn render(&self, template: &str, options: &::serialize_to_javascript::Options) -> ::serialize_to_javascript::Result<String> {
                        #replacements
                        Ok(template.into())
                    }
                }
            )
        }
        _ => {
            return syn::Error::new(
                item_span,
                "`Template` currently only supports data structs",
            )
                .to_compile_error()
                .into();
        }
    }
        .into()
}

#[proc_macro_attribute]
pub fn default_template(attr: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(attr as syn::LitStr);
    let item = parse_macro_input!(item as syn::DeriveInput);
    let name = item.ident.clone();
    let (impl_generics, ty_generics, _) = item.generics.split_for_impl();
    quote!(
        #item
        impl #impl_generics ::serialize_to_javascript::DefaultTemplate for #name #ty_generics {
            const RAW_TEMPLATE: &'static str = include_str!(#path);
        }
    )
    .into()
}
