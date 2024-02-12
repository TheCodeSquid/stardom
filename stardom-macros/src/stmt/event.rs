use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse::ParseStream, spanned::Spanned, Attribute, Expr, Ident, Meta, Token};

use super::{kw, StmtParse, StmtTokens};
use crate::util::*;

pub struct StmtEvent {
    pub attrs: Vec<Attribute>,
    pub key: Expr,
    pub value: Expr,
}

impl StmtParse for StmtEvent {
    fn parse_with_attrs(attrs: Vec<Attribute>, input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::on>()?;
        input.parse::<Token![:]>()?;
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;

        Ok(Self { attrs, key, value })
    }
}

impl StmtTokens for StmtEvent {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        let Paths {
            named,
            EventOptions,
            ..
        } = paths();

        let Self { attrs, key, value } = self;

        let mut once = None;
        let mut capture = None;
        let mut passive = None;

        for attr in attrs {
            let ident = attr.path().get_ident().map(|ident| ident.to_string());

            match ident.as_deref() {
                Some("once") => match flag(attr) {
                    Ok(expr) => once = Some(expr),
                    Err(err) => tokens.extend(err.to_compile_error()),
                },
                Some("capture") => match flag(attr) {
                    Ok(expr) => capture = Some(expr),
                    Err(err) => tokens.extend(err.to_compile_error()),
                },
                Some("passive") => match flag(attr) {
                    Ok(expr) => passive = Some(expr),
                    Err(err) => tokens.extend(err.to_compile_error()),
                },
                _ => tokens.extend(quote_spanned! {attr.span()=>
                    compile_error!("unrecognized attribute");
                }),
            }
        }

        let value_span = value.span().resolved_at(value.span());
        let f = if once.is_some() {
            // Wrap to accept FnOnce
            quote_spanned! {value_span=> {
                let mut __f = Some(#value);
                move |ev| {
                    (__f.take().expect("`once` event handler called more than once"))(ev);
                }
            }}
        } else {
            quote!(#value)
        };

        let once = once.map(|v| quote!(.once(#v)));
        let capture = capture.map(|v| quote!(.capture(#v)));
        let passive = passive.map(|v| quote!(.passive(#v)));

        let key_span = key.span().resolved_at(key.span());
        let key = quote_spanned! {key_span=> {
            #[allow(unused_imports)]
            use #named::events::*;
            #key
        }};

        tokens.extend(quote_spanned! {value_span=>
            #target.event(
                &#key,
                #EventOptions::new()
                    #once
                    #capture
                    #passive,
                #f
            );
        });
    }
}

fn flag(attr: &Attribute) -> syn::Result<Expr> {
    match &attr.meta {
        Meta::Path(_) => Ok(syn::parse_quote!(true)),
        Meta::List(meta) => Ok(meta.parse_args()?),
        Meta::NameValue(meta) => Ok(meta.value.clone()),
    }
}
