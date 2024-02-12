use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Expr, Ident, Token,
};

use crate::{
    stmt::{Stmt, StmtTokens},
    util::*,
};

pub struct Element {
    name: Expr,
    stmts: Vec<Stmt>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let stmts = if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            Stmt::parse_body(input)?
        } else {
            vec![]
        };

        Ok(Self { name, stmts })
    }
}

impl ToTokens for Element {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Paths { Node, .. } = paths();

        let Self { name, stmts } = self;
        let element = Ident::new("__element", Span::mixed_site());
        let stmts = stmts.apply_token_stream(&element);

        tokens.extend(quote! {{
            let #element = #Node::element(#name.into());
            #stmts
            #element
        }});
    }
}
