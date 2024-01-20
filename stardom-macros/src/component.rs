use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    ItemFn,
};

pub struct Component(ItemFn);

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let block = &self.0.block;

        let mut item_fn = self.0.clone();
        item_fn.block = syn::parse_quote_spanned! {block.span() => {
            stardom::node::Node::component(|| #block)
        }};

        item_fn.to_tokens(tokens);
    }
}
