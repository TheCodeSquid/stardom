use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Expr, ExprBlock, Ident, Local, LocalInit, Pat, PatType, Token,
};

use super::StmtTokens;

pub struct StmtLocal {
    local: Local,
}

impl Parse for StmtLocal {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;

        let let_token = input.parse()?;

        let mut pat = Pat::parse_single(input)?;
        if input.peek(Token![:]) {
            let colon_token = input.parse()?;
            let ty = input.parse()?;
            pat = Pat::Type(PatType {
                attrs: vec![],
                pat: Box::new(pat),
                colon_token,
                ty: Box::new(ty),
            });
        }

        let init = if input.peek(Token![=]) {
            let eq_token = input.parse()?;
            let expr = input.parse()?;

            let diverge = if input.peek(Token![else]) {
                let else_token = input.parse()?;
                let diverge = ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: input.parse()?,
                };
                Some((else_token, Box::new(Expr::Block(diverge))))
            } else {
                None
            };

            Some(LocalInit {
                eq_token,
                expr,
                diverge,
            })
        } else {
            None
        };

        let local = Local {
            attrs,
            let_token,
            pat,
            init,
            semi_token: Token![;](Span::call_site()),
        };
        Ok(Self { local })
    }
}

impl ToTokens for StmtLocal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.local.to_tokens(tokens);
    }
}

impl StmtTokens for StmtLocal {
    fn apply(&self, _target: &Ident, tokens: &mut TokenStream) {
        self.to_tokens(tokens);
    }
}
