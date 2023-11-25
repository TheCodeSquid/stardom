use proc_macro::TokenStream;
use syn::parse_macro_input;

mod dom;
mod generate;
mod parse;

use parse::NodeBodyMacro;

#[proc_macro]
pub fn node_body(input: TokenStream) -> TokenStream {
    let node_body = parse_macro_input!(input as NodeBodyMacro);

    generate::node_body(node_body).into()
}

#[proc_macro]
pub fn create_element_macros(_input: TokenStream) -> TokenStream {
    generate::tagged_macros().into()
}

#[proc_macro]
pub fn create_attributes(_input: TokenStream) -> TokenStream {
    generate::attributes().into()
}

#[proc_macro]
pub fn create_events(_input: TokenStream) -> TokenStream {
    generate::events().into()
}
