#![warn(clippy::use_self)]

mod generate;
mod parse;
mod web;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemFn};

use crate::parse::Element;

#[proc_macro_attribute]
pub fn component(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);

    let block = input.block.clone();
    input.block = Box::new(parse_quote! {
        {
            stardom::node::Node::component(move || #block)
        }
    });

    quote! { #input }.into()
}

#[proc_macro]
pub fn create_named_structures(_input: TokenStream) -> TokenStream {
    generate::create_named_structures().into()
}

#[proc_macro]
pub fn element(input: TokenStream) -> TokenStream {
    let element = parse_macro_input!(input as Element);
    generate::element(element).into()
}

// There are so many layers of code-generation to this.
// scrape.py -> include.rs -> define_tagged -> generate::element
// It's both beautiful and terrifying.

macro_rules! define_tagged {
    ($($tag:ident => ($lit:literal, $doc:literal),)*) => {
        $(
            #[doc = concat!("&lt;", $lit, "&gt;")]
            #[doc = "\n"]
            #[doc = concat!("[MDN Documentation](", $doc, ")")]
            #[proc_macro]
            pub fn $tag(input: TokenStream) -> TokenStream {
                let mut extended = TokenStream::new();
                extended.extend(TokenStream::from(quote! { $lit; }));
                extended.extend(input);
                let element = parse_macro_input!(extended as Element);
                generate::element(element).into()
            }
        )*
    };
}

macro_rules! define_reexport {
    ($($tag:ident,)*) => {
        #[proc_macro]
        pub fn reexport_elements(_input: TokenStream) -> TokenStream {
            quote! {
                pub mod elements {
                    pub use stardom_macros::{$($tag),*};
                }
            }.into()
        }
    };
}

include!("include.rs");
