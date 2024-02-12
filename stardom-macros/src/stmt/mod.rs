mod attr;
mod bind;
mod event;
mod local;
mod node;
mod reactive;

use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, Token,
};

pub use self::{
    attr::StmtAttr, bind::StmtBind, event::StmtEvent, local::StmtLocal, node::StmtNode,
    reactive::StmtReactive,
};

mod kw {
    syn::custom_keyword!(on);
    syn::custom_keyword!(bind);
}

trait StmtParse: Sized {
    fn parse_with_attrs(attrs: Vec<Attribute>, input: ParseStream) -> syn::Result<Self>;
}

pub trait StmtTokens {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream);

    fn apply_token_stream(&self, target: &Ident) -> TokenStream {
        let mut tokens = TokenStream::new();
        self.apply(target, &mut tokens);
        tokens
    }
}

pub enum Stmt {
    Local(StmtLocal),
    Node(StmtNode),
    Reactive(StmtReactive),
    Attr(StmtAttr),
    Event(StmtEvent),
    Bind(StmtBind),
}

impl Stmt {
    pub fn parse_body(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut stmts = vec![];
        let mut needs_separator = false;
        loop {
            let mut separated = false;
            while input.peek(Token![;]) {
                separated = true;
                input.parse::<Token![;]>()?;
            }
            if input.is_empty() {
                break;
            }
            if input.peek(Token![,]) {
                separated = true;
                input.parse::<Token![,]>()?;
            }

            if needs_separator && !separated {
                return Err(input.error("expected `,` or `;`"));
            }

            let stmt: Self = input.parse()?;
            needs_separator = stmt.needs_separator();
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn needs_separator(&self) -> bool {
        match self {
            Self::Local(_) | Self::Attr(_) | Self::Event(_) | Self::Bind(_) => true,
            Self::Node(node) => node.needs_separator(),
            Self::Reactive(reactive) => reactive.stmt.needs_separator(),
        }
    }
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = Attribute::parse_outer(input)?;

        if input.peek(Token![let]) {
            input.parse().map(Self::Local)
        } else if input.peek(Token![:]) {
            input.parse().map(Self::Reactive)
        } else if input.peek(kw::on) && input.peek2(Token![:]) {
            StmtParse::parse_with_attrs(attrs, input).map(Self::Event)
        } else if input.peek(kw::bind) && input.peek2(Token![:]) {
            StmtBind::parse_with_attrs(attrs, input).map(Self::Bind)
        } else if input.peek2(Token![=>]) {
            StmtAttr::parse_with_attrs(attrs, input).map(Self::Attr)
        } else {
            input.parse().map(Self::Node)
        }
    }
}

impl StmtTokens for Stmt {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        match self {
            Self::Local(stmt) => stmt.apply(target, tokens),
            Self::Node(stmt) => stmt.apply(target, tokens),
            Self::Reactive(stmt) => stmt.apply(target, tokens),
            Self::Attr(stmt) => stmt.apply(target, tokens),
            Self::Event(stmt) => stmt.apply(target, tokens),
            Self::Bind(stmt) => stmt.apply(target, tokens),
        }
    }
}

impl StmtTokens for Vec<Stmt> {
    fn apply(&self, target: &Ident, tokens: &mut TokenStream) {
        for stmt in self {
            stmt.apply(target, tokens);
        }
    }
}
