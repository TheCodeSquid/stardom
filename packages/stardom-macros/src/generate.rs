use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{
    parse::{NodeBodyMacro, NodeStmt},
    tagged::Dom,
};

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
            NodeStmt::Macro(mut expr) => {
                let tokens = expr.mac.tokens.clone();
                expr.mac.tokens = quote_spanned! {tokens.span() =>
                    #tokens ;* #target
                };

                quote_spanned! {expr.span() =>
                    #expr;
                }
            }
            NodeStmt::Child(expr) => {
                quote_spanned! {expr.span() => {
                    stardom_nodes::Node::insert(#target, &#expr, None);
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
                    let name = ::std::string::ToString::to_string(&#name);
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

pub fn tagged_macros(element_macro_path: syn::Path) -> TokenStream {
    let macros: Vec<_> = Dom::get()
        .elements
        .iter()
        .map(|elem| {
            let ident = syn::Ident::new(elem, Span::call_site());
            let lit = syn::LitStr::new(elem, Span::call_site());

            quote! {
                #[doc = concat!("&lt;", #lit, "&gt;")]
                #[macro_export]
                macro_rules! #ident {
                    ($($body:tt)*) => {
                        #element_macro_path!(#lit, $($body)*)
                    };
                }
            }
        })
        .collect();

    quote! { #(#macros)* }
}
