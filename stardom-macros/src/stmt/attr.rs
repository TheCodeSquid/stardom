use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::{parse::ParseStream, spanned::Spanned, Attribute, Expr, Ident, Token};

use crate::util::*;

use super::{StmtParse, StmtTokens};

pub struct StmtAttr {
    pub key: Expr,
    pub value: Expr,
}

impl StmtParse for StmtAttr {
    fn parse_with_attrs(attrs: Vec<Attribute>, input: ParseStream) -> syn::Result<Self> {
        if !attrs.is_empty() {
            return Err(syn::Error::new(
                attrs[0].span(),
                "attribute statements do not support macro attributes",
            ));
        }

        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}

impl StmtTokens for StmtAttr {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        let Paths {
            reactive,
            named,
            IntoAttr,
            ..
        } = paths();

        let Self { key, value } = self;

        let key_span = key.span().resolved_at(key.span());
        let key = quote_spanned! {key_span=> {
            #[allow(unused_imports)]
            use #named::attrs::*;
            ::std::borrow::Cow::<::std::primitive::str>::from(#key)
        }};

        let value_span = value.span().resolved_at(value.span());

        // There's no need for an effect if the value is a literal
        if matches!(value, Expr::Lit(_)) {
            tokens.extend(quote_spanned! {value_span=>
                #IntoAttr::set_attr(
                    #value,
                    &#target,
                    #key
                );
            })
        } else {
            tokens.extend(quote_spanned! {value_span=>
                #reactive::effect({
                    let #target = #target.clone();
                    move || #IntoAttr::set_attr(
                        #value,
                        &#target,
                        #key
                    )
                });
            });
        }
    }
}
