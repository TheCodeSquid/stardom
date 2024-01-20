use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token, Expr, ExprCall, Ident, Token,
};

use crate::{ident, join};

pub enum Stmt {
    Node(StmtNode),
    Dynamic(StmtNode),
    Parent(StmtParent),

    Attr(StmtAttr),
    Event(StmtEvent),
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Paren) {
            input.call(StmtNode::parse_parenthesized).map(Self::Node)
        } else if input.peek(token::Brace) {
            input.call(StmtNode::parse_braced).map(Self::Dynamic)
        } else if input.peek(Token![@]) {
            input.parse().map(Self::Event)
        } else if input.peek2(Token![=>]) {
            input.parse().map(Self::Attr)
        } else {
            let expr = input.parse()?;
            match expr {
                Expr::Call(call) if input.peek(token::Brace) => {
                    let content;
                    braced!(content in input);
                    let children = content.call(Self::parse_within)?;
                    Ok(Self::Parent(StmtParent { call, children }))
                }
                _ => Ok(Self::Node(StmtNode { expr })),
            }
        }
    }
}

impl Stmt {
    pub fn parse_within(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut stmts = vec![];
        loop {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                while input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                }
                if input.is_empty() {
                    break;
                }
            }

            stmts.push(input.parse()?);
        }
        Ok(stmts)
    }

    pub fn tokens_for_target(&self, target: &Ident) -> TokenStream {
        match self {
            Self::Node(StmtNode { expr }) => {
                let span = expr.span().resolved_at(expr.span());
                quote_spanned! {span=>
                    #target.insert(
                        &::std::convert::Into::into(#expr),
                        None
                    );
                }
            }
            Self::Dynamic(StmtNode { expr }) => {
                let span = expr.span().resolved_at(expr.span());
                quote_spanned! {span=>
                    let __current = stardom::node::Node::fragment();
                    #target.insert(&__current, None);
                    let __current = ::std::cell::Cell::new(__current);
                    stardom::reactive::effect({
                        let #target = #target.clone();
                        move || {
                            let __new: stardom::node::Node = ::std::convert::Into::into(#expr);
                            let __old = __current.replace(__new.clone());
                            #target.replace(&__old, &__new);
                        }
                    });
                }
            }
            Self::Parent(StmtParent { call, children }) => {
                let child_target = ident("__children");
                let child_stmts = stmts_to_tokens(&child_target, children);

                let mut call = call.clone();
                call.args.push(syn::parse_quote!(#child_target));

                let span = call.span().resolved_at(call.span());
                quote_spanned! {span=> {
                    const __CURRENT_NODE: stardom::util::AComponent = stardom::util::AComponent;
                    let #child_target = stardom::node::Node::fragment();
                    #child_stmts
                    #target.insert(&#call, None);
                }}
            }
            Self::Attr(StmtAttr { key, value }) => {
                let guard_span = join(key.span(), value.span());
                let guard = quote_spanned! {guard_span=>
                    let _: stardom::util::ThisNodeToBeAnElement = __CURRENT_NODE;
                };

                let value_span = value.span().resolved_at(value.span());
                let attr = quote_spanned! {value_span=>
                    let __attr = stardom::attr::Attr::from(#value);
                };

                let key_span = key.span().resolved_at(key.span());
                let apply = quote_spanned! {key_span=>
                    __attr.apply(&#target, {
                        #[allow(unused_imports)]
                        use stardom::util::attrs::*;
                        #key
                    });
                };

                quote! {
                    #guard
                    stardom::reactive::effect({
                        let #target = #target.clone();
                        move || {
                            #attr
                            #apply
                        }
                    });
                }
            }
            Self::Event(StmtEvent {
                at_token,
                key,
                value,
            }) => {
                let guard_span = join(at_token.span(), value.span());
                let guard = quote_spanned! {guard_span=>
                    let _: stardom::util::ThisNodeToBeAnElement = __CURRENT_NODE;
                };

                let key_span = key.span().resolved_at(key.span());
                let key_check = quote_spanned! {key_span=>
                    let _ = stardom::event::EventKey::name(&__key);
                };

                let value_span = value.span().resolved_at(value.span());
                let event = quote_spanned! {value_span=>
                    #target.event(
                        &__key,
                        stardom::event::EventOptions::default(),
                        #value
                    );
                };

                quote! {
                    #guard
                    let __key = {
                        #[allow(unused_imports)]
                        use stardom::util::events::*;
                        #key
                    };
                    #key_check
                    #event
                }
            }
        }
    }
}

pub fn stmts_to_tokens(target: &Ident, stmts: &[Stmt]) -> TokenStream {
    stmts
        .iter()
        .map(|stmt| stmt.tokens_for_target(target))
        .collect()
}

pub struct StmtNode {
    pub expr: Expr,
}

impl Parse for StmtNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            expr: input.parse()?,
        })
    }
}

impl StmtNode {
    fn parse_parenthesized(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(Self {
            expr: content.parse()?,
        })
    }

    fn parse_braced(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        Ok(Self {
            expr: content.parse()?,
        })
    }
}

pub struct StmtParent {
    pub call: ExprCall,
    pub children: Vec<Stmt>,
}

pub struct StmtAttr {
    pub key: Expr,
    pub value: Expr,
}

impl Parse for StmtAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;

        Ok(Self { key, value })
    }
}

pub struct StmtEvent {
    pub at_token: Token![@],
    pub key: Expr,
    pub value: Expr,
}

impl Parse for StmtEvent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let at_token = input.parse()?;
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;

        Ok(Self {
            at_token,
            key,
            value,
        })
    }
}
