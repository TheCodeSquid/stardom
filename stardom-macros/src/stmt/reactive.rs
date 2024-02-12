use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Ident, Token,
};

use super::{StmtNode, StmtTokens};
use crate::util::*;

pub struct StmtReactive {
    pub colon_token: Token![:],
    pub stmt: StmtNode,
}

impl Parse for StmtReactive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            colon_token: input.parse()?,
            stmt: input.parse()?,
        })
    }
}

impl StmtTokens for StmtReactive {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        let Paths {
            reactive,
            Node,
            IntoNode,
            ..
        } = paths();

        let stmt = &self.stmt;

        let span = stmt.span().resolved_at(stmt.span());
        tokens.extend(quote_spanned! {span=> {
            let mut __current = #Node::fragment();
            #target.insert(&__current, None);
            #reactive::effect({
                let #target = #target.clone();
                move || {
                    __current = #IntoNode::replace_self(#stmt, &#target, &__current);
                }
            });
        }});
    }
}
