use std::fmt::{self, Write};

use crate::{
    node::{Node, NodeKind},
    reactive::Runtime,
};

// Render

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

pub fn render_to_string<F>(f: F) -> String
where
    F: FnOnce() -> Node,
{
    let mut out = String::new();
    render(&mut out, f).unwrap();
    out
}

pub fn render<W, F>(w: &mut W, f: F) -> fmt::Result
where
    W: Write,
    F: FnOnce() -> Node,
{
    Runtime::init();

    let node = f();
    let state = node.0.state.borrow();
    match &state.kind {
        NodeKind::Text(content) => {
            w.write_str(&escape(content))?;
        }
        NodeKind::Element(elem) => {
            let attrs = elem
                .attrs
                .iter()
                .map(|(key, value)| format!(" {}=\"{}\"", key, escape(value)))
                .collect::<Vec<_>>()
                .join("");

            if state.children.is_empty() && VOID.contains(&elem.name.as_str()) {
                write!(w, "<{}{}>", elem.name, attrs)?;
            } else {
                write!(w, "<{}{}>", elem.name, attrs)?;
                for child in &state.children {
                    render(w, || child.clone())?;
                }
                write!(w, "</{}>", elem.name)?;
            }
        }
        NodeKind::Raw(content) => {
            w.write_str(content)?;
        }
        NodeKind::Opaque(_) => unreachable!(),
        NodeKind::Component(_) => {
            for child in &state.children {
                render(w, || child.clone())?;
            }
        }
    }

    Ok(())
}

fn escape(s: &str) -> String {
    let mut out = String::new();
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
