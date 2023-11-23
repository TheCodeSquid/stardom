use syn::parse_macro_input;

mod dom;
mod generate;
mod parse;

use parse::NodeBodyMacro;

#[proc_macro]
pub fn node_body(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let node_body = parse_macro_input!(input as NodeBodyMacro);

    generate::node_body(node_body).into()
}

#[proc_macro]
pub fn create_element_macros(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate::tagged_macros().into()
}

#[proc_macro]
pub fn create_attributes(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate::attributes().into()
}
