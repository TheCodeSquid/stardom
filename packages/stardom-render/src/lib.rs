mod node;

use std::{
    io::{self, Write},
    str,
};

use node::NodeKind;
pub use node::NodeRef;

// Reference: https://developer.mozilla.org/en-US/docs/Glossary/Void_element
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub fn render_string(node: &NodeRef) -> String {
    let mut buf = vec![];
    render(&mut buf, node).unwrap();
    // SAFETY: render can only produce valid UTF-8
    unsafe { String::from_utf8_unchecked(buf) }
}

pub fn render<W: Write>(mut w: W, node: &NodeRef) -> io::Result<()> {
    match &*node.kind() {
        NodeKind::Text(text) => {
            write!(w, "{}", escape(text))?;
        }
        NodeKind::Raw(raw) => {
            write!(w, "{raw}")?;
        }
        NodeKind::Fragment(children) => {
            write!(w, "{}", render_children(children, ""))?;
        }
        NodeKind::Element {
            namespace,
            name,
            attrs,
            children,
        } => {
            let tag = namespace
                .as_ref()
                .map(|ns| format!("{ns}:{name}"))
                .unwrap_or_else(|| name.clone());

            let attrs = attrs
                .iter()
                .map(|(name, value)| format!(" {name}=\"{}\"", escape(value)))
                .collect::<Vec<_>>()
                .join("");

            if !children.is_empty() {
                write!(
                    w,
                    "<{tag}{attrs}>\n{children}\n</{tag}>",
                    children = render_children(children, "  ")
                )?;
            } else if VOID_ELEMENTS.contains(&tag.as_str()) {
                write!(w, "<{tag}{attrs}>")?;
            } else {
                write!(w, "<{tag}{attrs}></{tag}>")?;
            }
        }
    }

    Ok(())
}

fn render_children(children: &[NodeRef], indent: &str) -> String {
    let mut buf = vec![];
    for child in children {
        render(&mut buf, child).unwrap();

        buf.push(b'\n');
    }

    // SAFETY: render can only produce valid UTF-8
    let text = unsafe { String::from_utf8_unchecked(buf) };
    text.lines()
        .map(|line| format!("{indent}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// Reference: https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html#output-encoding-for-html-contexts
pub fn escape(text: &str) -> String {
    let mut output = String::new();
    for c in text.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#x27;"),
            _ => output.push(c),
        }
    }
    output
}
