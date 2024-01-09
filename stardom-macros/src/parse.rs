use syn::{
    parse::{Parse, ParseStream},
    Token,
};

pub struct Element {
    pub name: syn::Expr,
    pub stmts: Vec<NodeStmt>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![;]>()?;

        let mut stmts = vec![];
        while !input.is_empty() {
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                continue;
            }

            stmts.push(input.parse()?);

            if !input.is_empty() && !input.peek(Token![;]) {
                return Err(input.error("';' expected"));
            }
        }

        Ok(Element { name, stmts })
    }
}

pub enum NodeStmt {
    Attr(syn::Expr, syn::Expr),
    Event(syn::Expr, syn::Expr),
    Node(syn::Expr),
}

impl Parse for NodeStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let key = input.parse()?;
            input.parse::<Token![=>]>()?;
            let f = input.parse()?;
            Ok(NodeStmt::Event(key, f))
        } else if input.peek2(Token![=>]) {
            let key = input.parse()?;
            input.parse::<Token![=>]>()?;
            let value = input.parse()?;
            Ok(NodeStmt::Attr(key, value))
        } else {
            let expr = input.parse()?;
            Ok(NodeStmt::Node(expr))
        }
    }
}
