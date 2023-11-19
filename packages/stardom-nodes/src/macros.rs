use stardom_macros::create_tagged_macros;

#[macro_export]
macro_rules! text_node {
    ($content:expr $(;* $parent:expr)?) => {{
        let text = stardom_nodes::Node::text();

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
macro_rules! raw_node {
    ($content:expr $(;* $parent:expr)?) => {{
        let raw = stardom_nodes::Node::raw();

        let r = ::std::clone::Clone::clone(&raw);
        stardom_reactive::effect(move || {
            let content = ::std::string::ToString::to_string(&$content);
            stardom_nodes::Node::set_text(&r, &content);
        });

        $(stardom_nodes::Node::insert($parent, &raw, None);)?
        raw
    }}
}

#[macro_export]
macro_rules! element {
    ($name:expr, $($body:tt)*) => {{
        let name = ::std::string::ToString::to_string(&$name);
        let element = stardom_nodes::Node::element(None, &name);

        let parent = stardom_macros::node_body!(&element, $($body)*);
        if let Some(parent) = parent {
            stardom_nodes::Node::insert(parent, &element, None);
        }

        element
    }};
}

#[macro_export]
macro_rules! fragment {
    ($($body:tt)*) => {{
        let fragment = stardom_nodes::Node::fragment();

        let parent = stardom_macros::node_body!(&fragment, $($body)*);
        if let Some(parent) = parent {
            stardom_nodes::Node::insert(parent, &fragment, None);
        }

        fragment
    }};
}

create_tagged_macros!(stardom_nodes::element);

#[macro_export]
macro_rules! show {
    ($condition:expr, { $($body:tt)* } ;* $parent:expr) => {{
        let fragment = stardom_nodes::Node::fragment();
        let child = stardom_nodes::Node::fragment();

        let _: Option<()> = stardom_macros::node_body!(&child, $($body)*);

        let f = ::std::clone::Clone::clone(&fragment);
        let last = ::std::cell::Cell::new(false);
        stardom_reactive::effect(move || {
            let now = $condition;
            if now == last.get() {
                return;
            } else {
                last.set(now);
            }

            if $condition {
                stardom_nodes::Node::insert(&f, &child, None);
            } else {
                stardom_nodes::Node::remove(&f, &child);
            }
        });

        stardom_nodes::Node::insert($parent, &fragment, None);
    }};
}
