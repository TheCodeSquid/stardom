use syn::{
    braced,
    parse::{Parse, ParseStream},
    token, Error, Result, Token,
};

pub struct NodeBodyMacro {
    pub target: syn::Expr,
    pub body: NodeBody,
}

impl Parse for NodeBodyMacro {
    fn parse(input: ParseStream) -> Result<Self> {
        let target = input.parse()?;
        input.parse::<Token![,]>()?;
        let body = input.parse()?;
        Ok(Self { target, body })
    }
}

pub struct NodeBody {
    pub stmts: Vec<NodeStmt>,
    pub parent: Option<syn::Expr>,
}

impl Parse for NodeBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stmts = vec![];
        let mut parent = None;

        while !input.is_empty() {
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                continue;
            }
            if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                parent = Some(input.parse()?);
                break;
            }

            stmts.push(input.parse()?);

            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                continue;
            } else if input.is_empty() {
                break;
            } else {
                return Err(syn::Error::new(input.span(), "expected `;`"));
            }
        }

        Ok(NodeBody { stmts, parent })
    }
}

pub enum NodeStmt {
    Macro(syn::ExprMacro),
    Child(syn::Expr),
    Fragment(Vec<NodeStmt>),
    Text(syn::Expr),
    Attr { name: syn::Expr, value: syn::Expr },
    Event { name: syn::Expr, f: syn::Expr },
}

impl Parse for NodeStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let stmt = if input.peek(token::Brace) {
            let body;
            braced!(body in input);

            let mut stmts = vec![];
            while !body.is_empty() {
                if input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                    continue;
                }

                stmts.push(input.parse()?);

                if input.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                    continue;
                } else if input.is_empty() {
                    break;
                } else {
                    return Err(syn::Error::new(input.span(), "expected `;`"));
                }
            }

            NodeStmt::Fragment(stmts)
        } else if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
            let expr = input.parse()?;

            NodeStmt::Text(expr)
        } else if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            let name = input.parse()?;
            input.parse::<Token![=>]>()?;
            let value = input.parse()?;

            NodeStmt::Attr { name, value }
        } else if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let name = input.parse()?;
            input.parse::<Token![=>]>()?;
            let f = input.parse()?;

            NodeStmt::Event { name, f }
        } else {
            let expr: syn::Expr = input.parse().map_err(|_| {
                Error::new(input.span(), "expected `+`, `#`, `@`, or an expression")
            })?;

            if let syn::Expr::Macro(expr) = expr {
                NodeStmt::Macro(expr)
            } else {
                NodeStmt::Child(expr)
            }
        };

        Ok(stmt)
    }
}
