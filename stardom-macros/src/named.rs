use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

use crate::util::*;

mod kw {
    syn::custom_keyword!(elements);
    syn::custom_keyword!(attributes);
    syn::custom_keyword!(events);
}

pub enum Named {
    Elements,
    Attributes,
    Events,
}

impl Parse for Named {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::elements) {
            input.parse::<kw::elements>()?;
            Ok(Self::Elements)
        } else if lookahead.peek(kw::attributes) {
            input.parse::<kw::attributes>()?;
            Ok(Self::Attributes)
        } else if input.peek(kw::events) {
            input.parse::<kw::events>()?;
            Ok(Self::Events)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for Named {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Paths {
            web_sys, EventKey, ..
        } = paths();

        tokens.extend(match self {
            Self::Elements => {
                let macros: TokenStream = crate::ELEMENTS
                    .iter()
                    .map(|element| {
                        let ident = Ident::new(element, Span::call_site());
                        quote!(#ident,)
                    })
                    .collect();

                quote! {
                    pub use stardom_macros::{#macros};
                }
            }
            Self::Attributes => crate::ATTRS
                .iter()
                .map(|attr| {
                    let snake = attr.to_case(Case::Snake);
                    let ident = Ident::new(&snake, Span::call_site());

                    quote! {
                        #[allow(non_upper_case_globals)]
                        pub const #ident: &str = #attr;
                    }
                })
                .collect(),
            Self::Events => crate::EVENTS
                .iter()
                .map(|(name, interface)| {
                    let name_ident = Ident::new(name, Span::call_site());
                    let camel = interface.to_case(Case::UpperCamel);
                    let interface_ident = Ident::new(&camel, Span::call_site());

                    quote! {
                        #[allow(non_camel_case_types)]
                        #[derive(Clone, Copy, Debug)]
                        pub struct #name_ident;
                        impl #EventKey for #name_ident {
                            type Event = #web_sys::#interface_ident;

                            fn name(&self) -> &str {
                                #name
                            }
                        }
                    }
                })
                .collect(),
        });
    }
}
