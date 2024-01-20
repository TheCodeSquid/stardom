use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};

use crate::ident;

mod kw {
    syn::custom_keyword!(elements);
    syn::custom_keyword!(attrs);
    syn::custom_keyword!(events);
}

const SKIP_ATTRS: &[&str] = &["as", "async", "for", "loop", "type"];
const SKIP_EVENTS: &[&str] = &["FormDataEvent", "ClipboardEvent"];

pub enum CreateNamed {
    Elements,
    Attrs,
    Events,
}

impl Parse for CreateNamed {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::elements) {
            input.parse::<kw::elements>()?;
            Ok(Self::Elements)
        } else if lookahead.peek(kw::attrs) {
            input.parse::<kw::attrs>()?;
            Ok(Self::Attrs)
        } else if lookahead.peek(kw::events) {
            input.parse::<kw::events>()?;
            Ok(Self::Events)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for CreateNamed {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let output: TokenStream = match self {
            Self::Elements => crate::ELEMENTS
                .iter()
                .map(|elem| {
                    let elem_ident = ident(elem);
                    quote! {
                        pub use stardom_macros::#elem_ident;
                    }
                })
                .collect(),
            Self::Attrs => crate::ATTRS
                .iter()
                .filter(|attr| !SKIP_ATTRS.contains(attr))
                .map(|attr| {
                    let attr_ident = ident(attr.to_case(Case::Snake));
                    quote! {
                        pub const #attr_ident: &str = #attr;
                    }
                })
                .collect(),
            Self::Events => crate::EVENTS
                .iter()
                .filter(|(_, interface)| !SKIP_EVENTS.contains(interface))
                .map(|(name, interface)| {
                    let name_ident = ident(name);
                    let interface_ident = ident(interface.to_case(Case::UpperCamel));

                    quote! {
                        pub struct #name_ident;
                        impl EventKey for #name_ident {
                            type Event = web_sys::#interface_ident;

                            fn name(&self) -> &str {
                                #name
                            }
                        }
                    }
                })
                .collect(),
        };

        tokens.extend(output);
    }
}
