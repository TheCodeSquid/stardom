use syn::parse_macro_input;

mod generate;
mod parse;
mod tagged;

use parse::NodeBodyMacro;

#[proc_macro]
pub fn node_body(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let node_body = parse_macro_input!(input as NodeBodyMacro);

    generate::node_body(node_body).into()
}

#[proc_macro]
pub fn create_tagged_macros(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let path = parse_macro_input!(input as syn::Path);

    generate::tagged_macros(path).into()
}
