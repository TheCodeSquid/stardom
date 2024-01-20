use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};

use super::stmt::{stmts_to_tokens, Stmt};
use crate::ident;

pub struct Fragment {
    stmts: Vec<Stmt>,
}

impl Parse for Fragment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            stmts: input.call(Stmt::parse_within)?,
        })
    }
}

impl ToTokens for Fragment {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target = ident("__fragment");
        let stmts = stmts_to_tokens(&target, &self.stmts);

        tokens.extend(quote! {{
            const __CURRENT_NODE: stardom::util::AFragmentNode = stardom::util::AFragmentNode;
            let #target = stardom::node::Node::fragment();
            #stmts
            #target
        }})
    }
}
