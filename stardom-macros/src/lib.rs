#![warn(clippy::use_self)]

mod bindings;
mod component;
mod element;
mod fragment;
mod named;
mod stmt;
mod util;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use self::{
    component::{Component, ComponentArgs},
    element::Element,
    fragment::Fragment,
    named::Named,
};

#[proc_macro]
pub fn create_named(input: TokenStream) -> TokenStream {
    let named = parse_macro_input!(input as Named);
    named.to_token_stream().into()
}

#[proc_macro]
pub fn fragment(input: TokenStream) -> TokenStream {
    let fragment = parse_macro_input!(input as Fragment);
    fragment.to_token_stream().into()
}

#[proc_macro]
pub fn element(input: TokenStream) -> TokenStream {
    let element = parse_macro_input!(input as Element);
    element.to_token_stream().into()
}

// Components

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ComponentArgs);

    let component = parse_macro_input!(input as Component);
    component.to_token_stream(args).into()
}

// Bindings

#[proc_macro]
pub fn bind_this(input: TokenStream) -> TokenStream {
    let bind = parse_macro_input!(input as bindings::BindThis);
    bindings::bind_this(bind).into()
}

#[proc_macro]
pub fn bind_value(input: TokenStream) -> TokenStream {
    let bind = parse_macro_input!(input as bindings::BindValue);
    bindings::bind_value(bind).into()
}

// Include definitions from stardom-codegen

macro_rules! define_elements {
    ($($name:ident,)*) => {
        $(
            #[proc_macro]
            pub fn $name(input: TokenStream) -> TokenStream {
                let name = stringify!($name);
                let mut extended = TokenStream::from(quote!(#name;));
                extended.extend(input);
                let element = parse_macro_input!(extended as Element);
                element.to_token_stream().into()
            }
        )*
    };
}

include!("./include.rs");
