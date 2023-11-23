use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{
    dom,
    parse::{NodeBodyMacro, NodeStmt},
};

const KEYWORDS: &[&str] = &["as", "async", "for", "loop", "type"];

pub fn node_body(NodeBodyMacro { target, body }: NodeBodyMacro) -> TokenStream {
    let stmts = create_stmts(&target, body.stmts);

    let parent = body
        .parent
        .map(|parent| {
            quote! {
                ::std::option::Option::Some(#parent)
            }
        })
        .unwrap_or(quote! { ::std::option::Option::None });

    quote! {{
        #(#stmts)*

        #parent
    }}
}

fn create_stmts(target: &syn::Expr, stmts: Vec<NodeStmt>) -> Vec<TokenStream> {
    stmts
        .into_iter()
        .map(|stmt| match stmt {
            NodeStmt::Child(expr) => {
                quote_spanned! {expr.span() => {
                    stardom_nodes::Node::insert(#target, &#expr, None);
                }}
            }
            NodeStmt::Macro(mut expr) => {
                let tokens = expr.mac.tokens.clone();
                expr.mac.tokens = quote_spanned! {tokens.span() =>
                    #tokens ;* #target
                };

                quote_spanned! {expr.span() =>
                    #expr;
                }
            }
            NodeStmt::Match(mut expr) => {
                for (i, arm) in expr.arms.iter_mut().enumerate() {
                    let body = arm.body.clone();
                    arm.body = syn::parse_quote_spanned! {body.span() => {
                        let node = arms
                            .entry(#i)
                            .or_insert_with(|| #body);
                        active.replace(::std::clone::Clone::clone(node))
                    }}
                }

                quote_spanned! {expr.span() => {
                    let fragment = stardom_nodes::Node::fragment();
                    let arms = ::std::cell::RefCell::new(::std::collections::HashMap::new());
                    let active = ::std::cell::RefCell::new(::std::option::Option::None);

                    let rt = stardom_reactive::Runtime::unwrap_global();
                    let scope = rt.active();

                    let frag = ::std::clone::Clone::clone(&fragment);
                    stardom_reactive::effect(move || {
                        let mut arms = arms.borrow_mut();
                        let mut active = active.borrow_mut();

                        let old = rt.with_parent(scope, || #expr);

                        if old == *active {
                            return;
                        }

                        if let Some(old) = old {
                            stardom_nodes::Node::remove(&frag, &old);
                        }
                        if let Some(active) = &*active {
                            stardom_nodes::Node::insert(&frag, active, None);
                        }
                    });

                    stardom_nodes::Node::insert(#target, &fragment, None);
                }}
            }
            NodeStmt::Fragment(brace, stmts) => {
                let fragment = syn::Ident::new("fragment", Span::call_site());
                let stmts = create_stmts(&syn::parse_quote!(&#fragment), stmts);

                quote_spanned! {brace.span => {
                    let parent = #target;
                    let #fragment = stardom_nodes::Node::fragment();

                    #(#stmts)*

                    stardom_nodes::Node::insert(parent, &#fragment, None);
                }}
            }
            NodeStmt::Text(expr) => {
                quote_spanned! {expr.span() => {
                    let parent = #target;
                    let text = stardom_nodes::Node::text();
                    stardom_nodes::Node::insert(parent, &text, None);

                    let t = ::std::clone::Clone::clone(&text);
                    stardom_reactive::effect(move || {
                        let value = ::std::string::ToString::to_string(&#expr);
                        stardom_nodes::Node::set_text(&t, &value);
                    });
                }}
            }
            NodeStmt::Attr { name, value } => {
                quote_spanned! {value.span() => {
                    let target = ::std::clone::Clone::clone(#target);
                    let name = {
                    use stardom_nodes::attributes::*;
                        ::std::string::ToString::to_string(&#name)
                    };
                    stardom_reactive::effect(move || {
                        let value = ::std::string::ToString::to_string(&#value);
                        stardom_nodes::Node::set_attr(&target, &name, &value);
                    });
                }}
            }
            NodeStmt::Event { name, f } => {
                quote_spanned! {f.span() => {
                    let name = ::std::string::ToString::to_string(&#name);
                    stardom_nodes::Node::event(#target, &name, #f);
                }}
            }
        })
        .collect()
}

pub fn tagged_macros() -> TokenStream {
    let macros: Vec<_> = dom::elements()
        .iter()
        .map(|elem| {
            let ident = syn::Ident::new(elem, Span::call_site());
            let lit = syn::LitStr::new(elem, Span::call_site());

            quote! {
                #[doc = concat!("&lt;", #lit, "&gt;")]
                #[macro_export]
                macro_rules! #ident {
                    ($($body:tt)*) => {
                        stardom_nodes::element!(#lit, $($body)*)
                    };
                }
            }
        })
        .collect();

    quote! { #(#macros)* }
}

pub fn attributes() -> TokenStream {
    let literals: Vec<_> = dom::attributes()
        .iter()
        .filter(|attr| !KEYWORDS.contains(&attr.as_str()))
        .map(|attr| {
            let snake = attr.to_case(Case::Snake);
            let ident = syn::Ident::new(&snake, Span::call_site());
            let lit = syn::LitStr::new(attr, Span::call_site());

            quote! {
                #[allow(non_upper_case_globals)]
                pub const #ident: &str = #lit;
            }
        })
        .collect();

    quote! { #(#literals)* }
}
