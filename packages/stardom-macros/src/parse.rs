use syn::{
    braced,
    parse::{Parse, ParseStream},
    token, Result, Token,
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
                return Err(input.error("expected `;`"));
            }
        }

        Ok(Self { stmts, parent })
    }
}

pub enum NodeStmt {
    Child(syn::Expr),
    Macro(syn::ExprMacro),
    Match(syn::ExprMatch),
    Fragment(token::Brace, Vec<NodeStmt>),
    Text(syn::Expr),
    Attr { name: syn::Expr, value: syn::Expr },
    Event { key: syn::Expr, f: syn::Expr },
}

impl Parse for NodeStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let stmt = if input.peek(token::Brace) {
            let body;
            let brace = braced!(body in input);

            let mut stmts = vec![];
            while !body.is_empty() {
                if body.peek(Token![;]) {
                    body.parse::<Token![;]>()?;
                    continue;
                }

                stmts.push(body.parse()?);

                if body.peek(Token![;]) {
                    body.parse::<Token![;]>()?;
                    continue;
                } else if body.is_empty() {
                    break;
                } else {
                    return Err(body.error("expected `;`"));
                }
            }

            Self::Fragment(brace, stmts)
        } else if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;
            let expr = input.parse()?;

            Self::Text(expr)
        } else if input.peek(Token![.]) {
            input.parse::<Token![.]>()?;
            let name = input.parse()?;
            input.parse::<Token![=>]>()?;
            let value = input.parse()?;

            Self::Attr { name, value }
        } else if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            let key = input.parse()?;
            input.parse::<Token![=>]>()?;
            let f = input.parse()?;

            Self::Event { key, f }
        } else if input.peek(Token![match]) {
            Self::Match(input.parse()?)
        } else {
            let expr: syn::Expr = input
                .parse()
                .map_err(|_| input.error("expected `{`, `+`, `.`, `@`, or an expression"))?;

            if let syn::Expr::Macro(expr) = expr {
                Self::Macro(expr)
            } else {
                Self::Child(expr)
            }
        };

        Ok(stmt)
    }
}
