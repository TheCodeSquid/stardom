use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

use crate::{
    stmt::{Stmt, StmtTokens},
    util::*,
};

pub struct Fragment {
    pub stmts: Vec<Stmt>,
}

impl Parse for Fragment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            stmts: Stmt::parse_body(input)?,
        })
    }
}

impl ToTokens for Fragment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Paths { Node, .. } = paths();

        let fragment = Ident::new("__fragment", Span::mixed_site());
        let stmts = self.stmts.apply_token_stream(&fragment);

        tokens.extend(quote! {{
            let #fragment = #Node::fragment();
            #stmts
            #fragment
        }});
    }
}
