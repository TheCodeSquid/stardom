use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    ItemFn,
};

use crate::util::*;

// TODO
pub struct ComponentArgs {}

impl Parse for ComponentArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(Self {})
    }
}

pub struct Component {
    item_fn: ItemFn,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            item_fn: input.parse()?,
        })
    }
}

impl Component {
    pub fn to_tokens(&self, _args: ComponentArgs, tokens: &mut TokenStream) {
        let Paths { Node, .. } = paths();

        let mut item_fn = self.item_fn.clone();
        let block = &item_fn.block;
        item_fn.block = syn::parse_quote_spanned! {block.span()=> {
            #Node::component(|| #block)
        }};

        tokens.extend(quote! {
            #item_fn
        });
    }

    pub fn to_token_stream(&self, args: ComponentArgs) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.to_tokens(args, &mut tokens);
        tokens
    }
}
