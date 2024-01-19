use syn::{
    braced,
    parse::{Parse, ParseBuffer, ParseStream, Result},
    token, Token,
};

pub struct Element {
    pub name: syn::Expr,
    pub stmts: Vec<Stmt>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;

        let stmts = if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            input.call(Stmt::parse_to_end)?
        } else {
            vec![]
        };

        Ok(Self { name, stmts })
    }
}

// TODO: move to individual structs / add attribute field

#[derive(Clone)]
pub enum Stmt {
    Event(StmtEvent),
    Attr(StmtAttr),
    Node(StmtNode),
    Reactive(StmtReactive),
}

#[derive(Clone)]
pub struct StmtEvent {
    pub attrs: Vec<syn::Attribute>,
    pub key: syn::Expr,
    pub value: syn::Expr,
}

#[derive(Clone)]
pub struct StmtAttr {
    pub attrs: Vec<syn::Attribute>,
    pub key: syn::Expr,
    pub value: syn::Expr,
}

#[derive(Clone)]
pub struct StmtNode {
    pub attrs: Vec<syn::Attribute>,
    pub expr: syn::Expr,
}

#[derive(Clone)]
pub struct StmtReactive {
    pub attrs: Vec<syn::Attribute>,
    pub expr: syn::Expr,
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {
        let begin = input.fork();
        input.call(syn::Attribute::parse_outer)?;

        if begin.peek(Token![@]) {
            input.parse().map(Self::Event)
        } else if begin.peek(token::Brace) {
            input.parse().map(Self::Reactive)
        } else if begin.peek2(Token![=>]) {
            input.parse().map(Self::Attr)
        } else {
            input.parse().map(Self::Node)
        }
    }
}

impl Stmt {
    pub fn parse_to_end(input: &ParseBuffer) -> Result<Vec<Self>> {
        let mut stmts = vec![];

        while !input.is_empty() {
            // Skip extra semicolons
            if input.peek(Token![;]) {
                input.parse::<Token![;]>()?;
                continue;
            }

            stmts.push(input.parse()?);

            // Require a semicolon if not empty
            if !input.is_empty() {
                input.parse::<Token![;]>()?;
            }
        }

        Ok(stmts)
    }
}

impl Parse for StmtEvent {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        input.parse::<Token![@]>()?;
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;

        Ok(Self { attrs, key, value })
    }
}

impl Parse for StmtAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;

        Ok(Self { attrs, key, value })
    }
}

impl Parse for StmtNode {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(syn::Attribute::parse_outer)?,
            expr: input.parse()?,
        })
    }
}

impl Parse for StmtReactive {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;

        let content;
        braced!(content in input);
        let expr = content.parse()?;

        Ok(Self { attrs, expr })
    }
}
