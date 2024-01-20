use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

use super::stmt::{stmts_to_tokens, Stmt};
use crate::ident;

pub struct Element {
    name: Expr,
    stmts: Vec<Stmt>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let stmts = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.call(Stmt::parse_within)?
        } else {
            vec![]
        };

        Ok(Self { name, stmts })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.name;
        let target = ident("__element");
        let stmts = stmts_to_tokens(&target, &self.stmts);

        tokens.extend(quote! {{
            const __CURRENT_NODE: stardom::util::ThisNodeToBeAnElement =
                stardom::util::ThisNodeToBeAnElement;
            let #target = if let Some((__ns, __name)) = #name.split_once(';') {
                stardom::node::Node::element_ns(__ns, __name)
            } else {
                stardom::node::Node::element(#name)
            };
            #stmts
            #target
        }});
    }
}
