use stardom_macros::create_tagged_macros;

#[macro_export]
macro_rules! text_node {
    ($content:expr $(;* $parent:expr)?) => {{
        let text = $crate::Node::text();

        let t = ::std::clone::Clone::clone(&text);
        stardom_reactive::effect(move || {
            let content = ::std::string::ToString::to_string(&$content);
            stardom_nodes::Node::set_text(&t, &content);
        });

        $(stardom_nodes::Node::insert($parent, &text, None);)?
        text
    }};
}

#[macro_export]
macro_rules! element {
    ($name:expr, $($body:tt)*) => {{
        let name = ::std::string::ToString::to_string(&$name);
        let element = $crate::Node::element(None, &name);

        let parent = stardom_macros::node_body!(&element, $($body)*);
        if let Some(parent) = parent {
            $crate::Node::insert(parent, &element, None);
        }

        element
    }};
}

#[macro_export]
macro_rules! fragment {
    ($($body:tt)*) => {{
        let fragment = $crate::Node::fragment();

        let parent = stardom_macros::node_body!(&fragment, $($body)*);
        if let Some(parent) = parent {
            $crate::Node::insert(parent, &fragment, None);
        }

        fragment
    }};
}

create_tagged_macros!(stardom_nodes::element);
