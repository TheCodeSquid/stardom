use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::ParseStream, spanned::Spanned, Attribute, ExprMacro, Ident, Token};

use super::{kw, StmtParse, StmtTokens};

pub struct StmtBind {
    pub bind: kw::bind,
    pub expr: ExprMacro,
}

impl StmtParse for StmtBind {
    fn parse_with_attrs(attrs: Vec<Attribute>, input: ParseStream) -> syn::Result<Self> {
        if !attrs.is_empty() {
            return Err(syn::Error::new(
                attrs[0].span(),
                "bindings do not support attributes",
            ));
        }

        let bind = input.parse()?;
        input.parse::<Token![:]>()?;
        let expr = input.parse()?;

        Ok(Self { bind, expr })
    }
}

impl StmtTokens for StmtBind {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        let mut expr = self.expr.clone();
        let mac_tokens = &mut expr.mac.tokens;
        *mac_tokens = quote! {
            #target; #mac_tokens
        };

        tokens.extend(quote! {{
            #expr
        }});
    }
}
