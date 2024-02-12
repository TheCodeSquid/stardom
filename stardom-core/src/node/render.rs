use std::fmt::{self, Write};

use bitflags::bitflags;

use crate::{
    env::{self, Env},
    node::{Node, NodeKind},
};

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Mode: u8 {
        const PRETTY = 0b01;
        const HYDRATION = 0b10;
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::PRETTY
    }
}

pub fn render_to_string<F>(mode: Mode, f: F) -> String
where
    F: FnOnce() -> Node,
{
    let mut out = String::new();
    render(&mut out, mode, f).unwrap();
    out
}

pub fn render<W, F>(w: &mut W, mode: Mode, f: F) -> fmt::Result
where
    W: Write,
    F: FnOnce() -> Node,
{
    env::with(Env::Render, || {
        stardom_reactive::run(|dispose| {
            let res = render_node(w, mode, &f());
            dispose();
            res
        })
    })
}

fn render_node<W: Write>(w: &mut W, mode: Mode, node: &Node) -> fmt::Result {
    let nl = |w: &mut W| {
        if mode.contains(Mode::PRETTY) {
            writeln!(w)
        } else {
            Ok(())
        }
    };

    match node.kind() {
        NodeKind::Element {
            name,
            namespace,
            attrs,
        } => {
            let full_name = if let Some(ns) = namespace {
                format!("{ns}:{name}")
            } else {
                name.clone()
            };

            let attr_str = attrs
                .borrow()
                .iter()
                .map(|(key, value)| {
                    let escaped = html_escape(value);
                    format!(" {}=\"{}\"", key, escaped)
                })
                .collect::<Vec<_>>()
                .join("");

            if VOID.contains(&name.as_str()) {
                write!(w, "<{full_name}{attr_str} />")?;
                nl(w)
            } else {
                write!(w, "<{full_name}{attr_str}>")?;
                nl(w)?;
                render_children(w, mode, true, node)?;
                write!(w, "</{full_name}>")?;
                nl(w)
            }
        }
        NodeKind::Text(content) => {
            let escaped = html_escape(&content.borrow());
            w.write_str(&escaped)?;
            nl(w)
        }
        NodeKind::Raw(content) => {
            if mode.contains(Mode::HYDRATION) {
                write!(
                    w,
                    "<!--stardom:raw-->{}<!--stardom:raw-->",
                    content.borrow()
                )?;
            } else {
                w.write_str(&content.borrow())?;
            }
            nl(w)
        }
        NodeKind::Fragment | NodeKind::Component(_) => render_children(w, mode, false, node),
    }
}

fn render_children<W: Write>(w: &mut W, mode: Mode, indent: bool, node: &Node) -> fmt::Result {
    if indent && mode.contains(Mode::PRETTY) {
        let mut buf = String::new();
        for child in &*node.children_ref() {
            render_node(&mut buf, mode, child).unwrap();
        }

        for line in buf.lines() {
            writeln!(w, "  {}", line)?;
        }
    } else {
        for child in &*node.children_ref() {
            render_node(w, mode, child)?;
        }
    }
    Ok(())
}

// See https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html#output-encoding-for-html-contexts
fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}
