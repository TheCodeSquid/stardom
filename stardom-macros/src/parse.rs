use syn::{
    braced,
    parse::{Parse, ParseBuffer, ParseStream},
    token, Token,
};

pub struct Element {
    pub name: syn::Expr,
    pub stmts: Vec<Stmt>,
}

impl Parse for Element {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![;]>()?;

        let stmts = parse_node_block(input)?;

        Ok(Self { name, stmts })
    }
}

pub fn parse_node_block(input: &ParseBuffer) -> syn::Result<Vec<Stmt>> {
    let mut stmts = vec![];
    while !input.is_empty() {
        if input.peek(Token![;]) {
            input.parse::<Token![;]>()?;
            continue;
        }

        stmts.push(input.parse()?);
    }
    Ok(stmts)
}

#[derive(Clone)]
pub enum Stmt {
    Node(StmtNode),
    Attr(StmtAttr),
    Event(StmtEvent),
    Fragment(StmtFragment),
    If(StmtIf),
    Match(StmtMatch),
}

#[derive(Clone)]
pub struct StmtNode {
    pub expr: syn::Expr,
}

#[derive(Clone)]
pub struct StmtAttr {
    pub key: syn::Expr,
    pub expr: syn::Expr,
}

#[derive(Clone)]
pub struct StmtEvent {
    pub key: syn::Expr,
    pub expr: syn::Expr,
}

#[derive(Clone)]
pub struct StmtFragment {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone)]
pub struct StmtIf {
    pub if_token: Token![if],
    pub cond: syn::Expr,
    pub then_branch: StmtFragment,
    pub else_branch: Option<(Token![else], Box<Stmt>)>,
}

#[derive(Clone)]
pub struct StmtMatch {
    pub match_token: Token![match],
    pub expr: syn::Expr,
    pub arms: Vec<Arm>,
}

#[derive(Clone)]
pub struct Arm {
    pub pat: syn::Pat,
    pub guard: Option<(Token![if], syn::Expr)>,
    pub fat_arrow_token: Token![=>],
    pub body: Box<Stmt>,
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace) {
            input.parse().map(Self::Fragment)
        } else if input.peek(Token![if]) {
            input.parse().map(Self::If)
        } else if input.peek(Token![match]) {
            input.parse().map(Self::Match)
        } else if input.peek(Token![@]) {
            input.parse().map(Self::Event)
        } else if input.peek2(Token![=>]) {
            input.parse().map(Self::Attr)
        } else {
            input.parse().map(Self::Node)
        }
    }
}

impl Parse for StmtNode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse()?;
        if !input.is_empty() {
            input.parse::<Token![;]>()?;
        }

        Ok(Self { expr })
    }
}

impl Parse for StmtAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let expr = input.parse()?;
        Ok(Self { key, expr })
    }
}

impl Parse for StmtEvent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![@]>()?;
        let key = input.parse()?;
        input.parse::<Token![=>]>()?;
        let expr = input.parse()?;
        Ok(Self { key, expr })
    }
}

impl Parse for StmtFragment {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        let stmts = parse_node_block(&content)?;
        Ok(Self { stmts })
    }
}

impl Parse for StmtIf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let if_token = input.parse()?;
        let cond = input.call(syn::Expr::parse_without_eager_brace)?;

        let branch;
        braced!(branch in input);
        let stmts = parse_node_block(&branch)?;
        let then_branch = StmtFragment { stmts };

        let else_branch = if input.peek(Token![else]) {
            Some(input.call(else_block)?)
        } else {
            None
        };

        Ok(Self {
            if_token,
            cond,
            then_branch,
            else_branch,
        })
    }
}

impl Parse for StmtMatch {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let match_token = input.parse()?;
        let expr = input.call(syn::Expr::parse_without_eager_brace)?;

        let content;
        braced!(content in input);

        let mut arms = vec![];
        while !content.is_empty() {
            arms.push(content.parse()?);
        }

        Ok(Self {
            match_token,
            expr,
            arms,
        })
    }
}

impl Parse for Arm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            pat: syn::Pat::parse_multi_with_leading_vert(input)?,
            guard: {
                if input.peek(Token![if]) {
                    let if_token = input.parse()?;
                    let guard = input.parse()?;
                    Some((if_token, guard))
                } else {
                    None
                }
            },
            fat_arrow_token: input.parse()?,
            body: input.parse()?,
        })
    }
}

fn else_block(input: ParseStream) -> syn::Result<(Token![else], Box<Stmt>)> {
    let else_token = input.parse()?;

    let lookahead = input.lookahead1();

    if !lookahead.peek(Token![if]) && !lookahead.peek(token::Brace) {
        return Err(lookahead.error());
    }
    let else_branch = input.parse()?;

    Ok((else_token, else_branch))
}
