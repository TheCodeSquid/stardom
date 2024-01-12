use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::{parse::*, web};

const KEYWORDS: &[&str] = &["as", "async", "for", "loop", "type"];
const MISSING: &[&str] = &[
    "ToggleEvent",
    "NavigationCurrentEntryChangeEvent",
    "FormDataEvent",
    "NavigateEvent",
    "PageRevealEvent",
];

pub fn create_named_structures() -> TokenStream {
    let attrs = web::ATTRS.iter().filter_map(|attr| {
        let snake = attr.to_case(Case::Snake);
        if KEYWORDS.contains(&snake.as_str()) {
            return None;
        }

        let ident = syn::Ident::new(&snake, Span::call_site());
        let lit = syn::LitStr::new(attr, Span::call_site());
        Some(quote! {
            #[doc = concat!("`", #lit, "` attribute")]
            #[allow(non_upper_case_globals)]
            pub const #ident: &str = #lit;
        })
    });

    let events = web::EVENTS.iter().map(|(name, interface)| {
        let name_ident = syn::Ident::new(name, Span::call_site());
        let name_lit = syn::LitStr::new(name, Span::call_site());

        let interface_camel = interface.to_case(Case::UpperCamel);
        let interface = syn::Ident::new(
            if MISSING.contains(&interface_camel.as_str()) {
                "Event"
            } else {
                &interface_camel
            },
            Span::call_site(),
        );

        quote! {
            #[doc = concat!("`", #name_lit, "` event")]
            #[allow(non_camel_case_types)]
            pub struct #name_ident;
            impl crate::EventKey for #name_ident {
                type Value = web_sys::#interface;

                fn name(&self) -> &str {
                    #name_lit
                }
            }
        }
    });

    quote! {
        pub mod attrs {
            #(#attrs)*
        }

        pub mod events {
            #(#events)*
        }
    }
}

pub fn element(Element { name, stmts }: Element) -> TokenStream {
    let node = syn::Ident::new("__node", Span::call_site());

    let stmts: Vec<_> = stmts.iter().map(|stmt| statement(stmt, &node)).collect();

    quote! {
        {
            let #node = stardom::node::Node::element(None, ::std::convert::Into::into(#name));

            {#(#stmts)*}

            #node
        }
    }
}

fn statement(stmt: &Stmt, target: &syn::Ident) -> TokenStream {
    match stmt {
        Stmt::Node(StmtNode { expr }) => {
            quote! {
                stardom::node::CaptureNode::capture_node(&#target, move || #expr);
            }
        }
        Stmt::Attr(StmtAttr { key, expr }) => {
            quote! {
                let __key: ::std::string::String = {
                    use stardom::util::attrs::*;
                    ::std::convert::Into::into(#key)
                };
                let __value: ::std::string::String = ::std::convert::Into::into(#expr);
                #target.set_attr(__key, __value);
            }
        }
        Stmt::Event(StmtEvent { key, expr }) => {
            quote! {
                #target.event(
                    {
                        use stardom::util::events::*;
                        #key
                    },
                    false,
                    #expr
                );
            }
        }
        Stmt::Fragment(StmtFragment { stmts }) => {
            let fragment = syn::Ident::new("__fragment", Span::call_site());
            let stmts: Vec<_> = stmts
                .iter()
                .map(|stmt| statement(stmt, &fragment))
                .collect();

            quote! {
                #target.insert(&{
                    let #fragment = stardom::node::Node::fragment();
                    { #(#stmts)* }
                    #fragment
                }, None);
            }
        }

        // Limitations of if/match node statements:
        // - all bound variables are tracked
        // -
        //
        // To avoid these conditions, use (if/match) with explicit parenthesis.
        Stmt::If(StmtIf {
            if_token,
            cond,
            then_branch,
            else_branch,
        }) => {
            let mut chain = vec![(quote!(#if_token), Some(cond.clone()), then_branch.clone())];

            let mut unclosed = true;
            let mut branch = else_branch.clone();
            while let Some((else_token, stmt)) = branch.take() {
                match *stmt {
                    Stmt::Fragment(frag) => {
                        unclosed = false;
                        chain.push((quote! { #else_token }, None, frag));
                    }
                    Stmt::If(StmtIf {
                        if_token,
                        cond,
                        then_branch,
                        else_branch,
                    }) => {
                        chain.push((quote!(#else_token #if_token), Some(cond), then_branch));
                        branch = else_branch;
                    }
                    _ => unreachable!(),
                }
            }

            if unclosed {
                chain.push((quote!(else), None, StmtFragment { stmts: vec![] }));
            }

            let signal = syn::Ident::new("__signal", Span::call_site());
            let current = syn::Ident::new("__current", Span::call_site());
            let (init, branches): (Vec<_>, Vec<_>) = chain
                .iter()
                .enumerate()
                .map(|(i, (prefix, cond, branch))| {
                    let (bind, rebind) =
                        if let Some(syn::Expr::Let(syn::ExprLet { pat, .. })) = cond {
                            rebind_pat(pat)
                        } else {
                            (quote!(), quote!())
                        };

                    let holder = syn::Ident::new(&format!("__b{i}"), Span::call_site());
                    let frag = syn::Ident::new("__branch", Span::call_site());
                    let stmt = statement(&Stmt::Fragment(branch.clone()), &frag);

                    (
                        quote! {
                            let mut #holder = None;
                            #bind
                        },
                        quote! {
                            #prefix #cond {
                                #rebind
                                let __b = #holder.get_or_insert_with(move || {
                                    let #frag = stardom::node::Node::fragment();
                                    #stmt
                                    #frag
                                });
                                #target.replace(&#current, __b);
                                #current = __b.clone();
                            }
                        },
                    )
                })
                .unzip();

            quote! {
                let #signal = stardom::reactive::signal(());
                let mut #current = stardom::node::Node::fragment();
                #target.insert(&#current, None);
                #(#init)*

                stardom::reactive::effect({
                    let #target = #target.clone();
                    move || {
                        stardom::reactive::Output::trigger(&#signal);
                        #(#branches)*
                    }
                });
            }
        }
        Stmt::Match(StmtMatch {
            match_token,
            expr,
            arms,
        }) => {
            let current = syn::Ident::new("__current", Span::call_site());
            let (init, arms): (Vec<_>, Vec<_>) = arms
                .iter()
                .enumerate()
                .map(
                    |(
                        i,
                        Arm {
                            pat,
                            guard,
                            fat_arrow_token,
                            body,
                        },
                    )| {
                        let (bind, rebind) = rebind_pat(pat);

                        let guard = guard
                            .as_ref()
                            .map(|(if_token, expr)| quote!(#if_token #expr));

                        let holder = syn::Ident::new(&format!("__a{i}"), Span::call_site());
                        let frag = syn::Ident::new("__frag", Span::call_site());
                        let stmt = statement(body, &frag);

                        (
                            quote! {
                                let mut #holder = None;
                                #bind
                            },
                            quote! {
                                #pat #guard #fat_arrow_token {
                                    #rebind
                                    let __a = #holder
                                        .get_or_insert_with(|| {
                                            let #frag = stardom::node::Node::fragment();
                                            #stmt
                                            #frag
                                        });
                                    #target.replace(&#current, &__a);
                                    #current = __a.clone();
                                }
                            },
                        )
                    },
                )
                .unzip();

            quote! {
                let mut #current = stardom::node::Node::fragment();
                #target.insert(&#current, None);
                #(#init)*

                stardom::reactive::effect({
                    let #target = #target.clone();
                    move || {
                        #match_token #expr {
                            #(#arms)*
                        }
                    }
                });
            }
        }
    }
}

fn rebind_pat(pat: &syn::Pat) -> (TokenStream, TokenStream) {
    pat_idents(pat)
        .into_iter()
        .enumerate()
        .map(|(i, ident)| {
            let holder = syn::Ident::new(&format!("__s{i}_{ident}"), Span::call_site());

            (
                quote! {
                    let mut #holder = None;
                },
                quote! {
                    let #ident = if let Some(__signal) = #holder {
                        stardom::reactive::Output::set(&__signal, #ident);
                        __signal
                    } else {
                        let __signal = stardom::reactive::signal(#ident);
                        #holder = Some(__signal);
                        __signal
                    };
                },
            )
        })
        .unzip()
}

fn pat_idents(pat: &syn::Pat) -> Vec<&syn::Ident> {
    match pat {
        syn::Pat::Ident(syn::PatIdent { ident, subpat, .. }) => {
            let mut out = vec![ident];
            if let Some((_, sub)) = subpat {
                out.append(&mut pat_idents(sub));
            }
            out
        }
        syn::Pat::Or(syn::PatOr { cases, .. }) => {
            cases.iter().flat_map(|sub| pat_idents(sub)).collect()
        }
        syn::Pat::Paren(syn::PatParen { pat, .. }) => pat_idents(pat),
        syn::Pat::Slice(syn::PatSlice { elems, .. }) => {
            elems.iter().flat_map(|sub| pat_idents(sub)).collect()
        }
        syn::Pat::Struct(syn::PatStruct { fields, .. }) => fields
            .iter()
            .flat_map(|field| pat_idents(&field.pat))
            .collect(),
        syn::Pat::Tuple(syn::PatTuple { elems, .. }) => {
            elems.iter().flat_map(|sub| pat_idents(sub)).collect()
        }
        syn::Pat::TupleStruct(syn::PatTupleStruct { elems, .. }) => {
            elems.iter().flat_map(|sub| pat_idents(sub)).collect()
        }
        syn::Pat::Type(syn::PatType { pat, .. }) => pat_idents(pat),

        syn::Pat::Const(_)
        | syn::Pat::Lit(_)
        | syn::Pat::Macro(_)
        | syn::Pat::Path(_)
        | syn::Pat::Range(_)
        | syn::Pat::Reference(_)
        | syn::Pat::Rest(_)
        | syn::Pat::Verbatim(_)
        | syn::Pat::Wild(_) => vec![],
        _ => unimplemented!(),
    }
}
