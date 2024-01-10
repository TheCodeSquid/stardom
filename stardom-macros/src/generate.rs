#![warn(clippy::use_self)]

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{
    parse::{Element, NodeStmt},
    web,
};

const KEYWORDS: &[&str] = &["as", "async", "for", "loop", "type"];
const MISSING: &[&str] = &[
    "ToggleEvent",
    "NavigationCurrentEntryChangeEvent",
    "FormDataEvent",
    "NavigateEvent",
    "PageRevealEvent",
];

pub fn create_named_structures() -> TokenStream {
    let elements = web::ELEMENTS.iter().map(|elem| {
        let ident = syn::Ident::new(elem, Span::call_site());
        let lit = syn::LitStr::new(elem, Span::call_site());

        quote! {
            #[macro_export]
            macro_rules! #ident {
                ($($body:tt)*) => {
                    stardom::element!(#lit; $($body)*)
                };
            }
        }
    });

    let attrs = web::ATTRS.iter().filter_map(|attr| {
        let snake = attr.to_case(Case::Snake);
        if KEYWORDS.contains(&snake.as_str()) {
            return None;
        }

        let ident = syn::Ident::new(&snake, Span::call_site());
        let lit = syn::LitStr::new(attr, Span::call_site());
        Some(quote! {
            #[allow(non_upper_case_globals)]
            pub const #ident: &str = #lit;
        })
    });

    let events = web::EVENTS.iter().map(|(name, interface)| {
        let name_ident = syn::Ident::new(name, Span::call_site());
        let name_lit = syn::LitStr::new(name, Span::call_site());

        let interface_camel = interface.to_case(Case::UpperCamel);
        let interface = syn::Ident::new(
            if MISSING.contains(&interface_camel.as_str()) {
                "Event"
            } else {
                &interface_camel
            },
            Span::call_site(),
        );

        quote! {
            #[allow(non_camel_case_types)]
            pub struct #name_ident;
            impl crate::EventKey for #name_ident {
                type Value = web_sys::#interface;

                fn name(&self) -> &str {
                    #name_lit
                }
            }
        }
    });

    quote! {
        #(#elements)*

        pub mod attrs {
            #(#attrs)*
        }

        pub mod events {
            #(#events)*
        }
    }
}

pub fn element(Element { name, stmts }: Element) -> TokenStream {
    let node = syn::Ident::new("node", Span::call_site());

    let stmts: Vec<_> = stmts
        .into_iter()
        .map(|stmt| statement(stmt, &node))
        .collect();

    quote! {
        {
            let #node = stardom::Node::element(None, ::std::convert::Into::into(#name));

            {#(#stmts)*}

            #node
        }
    }
}

fn statement(stmt: NodeStmt, target: &syn::Ident) -> TokenStream {
    match stmt {
        NodeStmt::Attr(key, value) => {
            quote! {
                let key: ::std::string::String = {
                    use stardom::constants::attrs::*;
                    ::std::convert::Into::into(#key)
                };
                let value: ::std::string::String = ::std::convert::Into::into(#value);
                #target.set_attr(key, value);
            }
        }
        NodeStmt::Event(key, f) => {
            quote! {
                let key = {
                    use stardom::constants::events::*;
                    #key
                };
                let name = stardom::EventKey::name(&key).to_string();
                #target.event(
                    key,
                    false,
                    #f
                );
            }
        }
        NodeStmt::Node(expr) => {
            quote! {
                #target.insert(&::std::convert::Into::into(#expr), None);
            }
        }
    }
}
