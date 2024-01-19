use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;

use crate::parse::*;

const RESERVED: &[&str] = &["as", "async", "for", "loop", "type"];

const MISSING: &[&str] = &["FormDataEvent"];
const UNSTABLE: &[&str] = &["ClipboardEvent"];

pub fn create_named() -> TokenStream {
    let elements: Vec<_> = crate::ELEMENTS
        .iter()
        .map(|element| syn::Ident::new(element, Span::call_site()))
        .collect();

    let attrs: Vec<_> = crate::ATTRS
        .iter()
        .filter_map(|attr| {
            if RESERVED.contains(attr) {
                return None;
            }

            let lit = syn::LitStr::new(attr, Span::call_site());
            let snake = attr.to_case(Case::Snake);
            let attr = syn::Ident::new(&snake, Span::call_site());

            Some(quote! {
                #[doc = concat!("`", #lit, "` attribute")]
                pub const #attr: &str = #lit;
            })
        })
        .collect();

    let events: Vec<_> = crate::EVENTS
        .iter()
        .map(|(event, interface)| {
            let is_unstable = UNSTABLE.contains(interface);

            let lit = syn::LitStr::new(event, Span::call_site());
            let ident = syn::Ident::new(event, Span::call_site());
            let interface = syn::Ident::new(
                &if MISSING.contains(interface) {
                    "Event"
                } else {
                    interface
                }.to_case(Case::UpperCamel),
                Span::call_site(),
            );

            let unstable = is_unstable.then(|| {
                quote! {
                    #[cfg(web_sys_unstable_apis)]
                }
            });
            let unstable_doc = is_unstable.then(|| {
                quote! {
                    #[doc = "\n"]
                    #[doc = "*The underlying `web-sys` API is unstable, so this requires [`--cfg=web_sys_unstable_apis`](https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html)*"]
                }
            });

            quote! {
                #unstable
                #[doc = concat!("`", #lit, "` event")]
                #unstable_doc
                #[derive(Clone, Copy, Debug)]
                pub struct #ident;
                #unstable
                impl EventKey for #ident {
                    type Event = web_sys::#interface;

                    fn name(&self) -> &str {
                        #lit
                    }
                }
            }
        })
        .collect();

    quote! {
        pub mod element {
            pub use stardom_macros::{#(#elements),*};
        }

        #[allow(non_upper_case_globals)]
        pub mod attr {
            use crate::node::Node;

            pub trait Attr {
                fn apply_attr(self, node: &Node, key: impl Into<String>);
            }

            impl Attr for &str {
                fn apply_attr(self, n: &Node, k: impl Into<String>) {
                    n.set_attr(k, self);
                }
            }

            impl Attr for String {
                fn apply_attr(self, n: &Node, k: impl Into<String>) {
                    n.set_attr(k, self);
                }
            }

            impl Attr for Option<&str> {
                fn apply_attr(self, n: &Node, k: impl Into<String>) {
                    if let Some(value) = self {
                        n.set_attr(k, value);
                    } else {
                        n.remove_attr(k);
                    }
                }
            }

            impl Attr for Option<String> {
                fn apply_attr(self, n: &Node, k: impl Into<String>) {
                    if let Some(v) = self {
                        n.set_attr(k, v);
                    } else {
                        n.remove_attr(k);
                    }
                }
            }

            #(#attrs)*
        }

        #[allow(non_camel_case_types)]
        pub mod event {
            pub trait EventKey {
                type Event: wasm_bindgen::JsCast;

                fn name(&self) -> &str;
            }

            #(#events)*
        }
    }
}

pub fn component(mut input: syn::ItemFn) -> TokenStream {
    let block = &input.block;

    input.block = syn::parse_quote_spanned! {block.span() => {
        stardom::node::Node::component(|| #block)
    }};

    quote! { #input }
}

pub fn element(Element { name, stmts }: Element) -> TokenStream {
    let node = syn::Ident::new("__target", Span::call_site());

    let stmts: Vec<_> = stmts.iter().map(|stmt| statement(&node, stmt)).collect();

    quote! {{
        let #node = {
            let __name = #name;
            if let Some((__ns, __name)) = __name.split_once(':') {
                stardom::node::Node::element_ns(__ns, __name)
            } else {
                stardom::node::Node::element(__name)
            }
        };
        #(#stmts)*
        #node
    }}
}

fn statement(target: &syn::Ident, stmt: &Stmt) -> TokenStream {
    match stmt {
        Stmt::Event(StmtEvent { attrs, key, value }) => quote! {
            #(#attrs)*
            #target.event({
                use stardom::util::event::*;
                #key
            }, false, #value);
        },
        Stmt::Attr(StmtAttr { attrs, key, value }) => {
            quote! {
                stardom::reactive::effect({
                    let #target = #target.clone();
                    move || {
                        #(#attrs)*
                        stardom::util::attr::Attr::apply_attr(
                            #value,
                            &#target,
                            {
                                use stardom::util::attr::*;
                                #key
                            }
                        );
                    }
                });
            }
        }
        Stmt::Node(StmtNode { attrs, expr }) => {
            quote! {
                #(#attrs)*
                #target.insert(
                    &::std::convert::Into::into(#expr),
                    None
                );
            }
        }
        Stmt::Reactive(StmtReactive { attrs, expr }) => {
            quote! {{
                let __slot = stardom::node::Node::fragment();
                #target.insert(&__slot, None);
                let __slot = ::std::cell::Cell::new(__slot);
                stardom::reactive::effect({
                    let #target = #target.clone();
                    move || {
                        let __new = stardom::node::Node::from(
                            #[allow(unused_braces)]
                            #(#attrs)*
                            #expr
                        );
                        let __old = __slot.replace(__new.clone());
                        #target.replace(&__old, &__new);
                    }
                });
            }}
        }
    }
}
