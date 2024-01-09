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
            stardom::Node::component(|| #block)
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
    let input = parse_macro_input!(input as Element);
    generate::element(input).into()
}
