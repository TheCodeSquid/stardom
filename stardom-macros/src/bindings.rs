use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Expr, Ident, Token,
};

use crate::util::*;

pub struct Binding<T: Parse> {
    pub target: Ident,
    pub body: T,
}

impl<T: Parse> Parse for Binding<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let target = input.parse()?;
        input.parse::<Token![;]>()?;
        let body = input.parse()?;

        Ok(Self { target, body })
    }
}

pub type BindThis = Binding<Expr>;
pub fn bind_this(binding: BindThis) -> TokenStream {
    let Paths { NodeRef, .. } = paths();

    let Binding { target, body } = binding;

    let span = body.span().resolved_at(body.span());
    quote_spanned! {span=> {
        #NodeRef::set(
            &#body,
            ::std::clone::Clone::clone(&#target)
        );
    }}
}

pub type BindValue = Binding<Expr>;
pub fn bind_value(binding: BindValue) -> TokenStream {
    let Paths { bindings, Node, .. } = paths();

    let Binding { target, body } = binding;

    let span = body.span().resolved_at(body.span());
    quote_spanned! {span=> {
        match #Node::element_name(&#target) {
            "input" | "textarea" => {
                #bindings::bind_value(&#target, #body);
            }
            "select" => {
                todo!("select");
            }
            name => {
                panic!("element `{name}` unsupported by binding `value`")
            }
        }
    }}
}
