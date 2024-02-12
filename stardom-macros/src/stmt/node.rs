use std::iter;

use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::{
    braced,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, Expr, ExprCall, ExprMacro, ExprMethodCall, Ident, Macro, MacroDelimiter,
};

use super::{Stmt, StmtTokens};
use crate::util::*;

pub struct StmtNode {
    pub expr: Expr,
}

impl StmtNode {
    pub fn needs_separator(&self) -> bool {
        !matches!(
            &self.expr,
            Expr::If(_)
                | Expr::Match(_)
                | Expr::Block(_)
                | Expr::Unsafe(_)
                | Expr::While(_)
                | Expr::Loop(_)
                | Expr::ForLoop(_)
                | Expr::TryBlock(_)
                | Expr::Const(_)
                | Expr::Macro(ExprMacro {
                    mac: Macro {
                        delimiter: MacroDelimiter::Brace(_),
                        ..
                    },
                    ..
                })
        )
    }
}

impl Parse for StmtNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut expr = Expr::parse_without_eager_brace(input)?;
        if input.peek(token::Brace) {
            match &mut expr {
                Expr::Call(ExprCall { args, .. })
                | Expr::MethodCall(ExprMethodCall { args, .. }) => {
                    inject_call_children(input, args)?;
                }
                _ => {}
            }
        }

        Ok(Self { expr })
    }
}

impl ToTokens for StmtNode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.expr.to_tokens(tokens);
    }
}

impl StmtTokens for StmtNode {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        let Paths { IntoNode, .. } = paths();

        let expr = &self.expr;

        let span = expr.span().resolved_at(expr.span());
        tokens.extend(quote_spanned! {span=>
            #target.insert(
                &#IntoNode::into_node(#expr),
                None
            );
        });
    }
}

fn inject_call_children<T>(input: ParseStream, args: &mut T) -> syn::Result<()>
where
    T: Extend<Expr>,
{
    let Paths { Node, .. } = paths();

    let content;
    let brace_token = braced!(content in input);
    let stmts = Stmt::parse_body(&content)?;

    let children = Ident::new("__children", Span::mixed_site());
    let stmts = stmts.apply_token_stream(&children);

    let expr = syn::parse_quote_spanned! {brace_token.span.join()=> {
        let #children = #Node::fragment();
        #stmts
        #children
    }};
    args.extend(iter::once(expr));
    Ok(())
}
