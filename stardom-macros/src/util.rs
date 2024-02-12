use syn::{parse_quote as q, Path};

#[allow(non_snake_case)]
#[derive(Clone)]
pub struct Paths {
    pub reactive: Path,
    pub macros: Path,
    pub named: Path,
    pub bindings: Path,
    pub web_sys: Path,

    pub Node: Path,
    pub IntoNode: Path,
    pub NodeRef: Path,

    pub IntoAttr: Path,
    pub EventKey: Path,
    pub EventOptions: Path,
}

impl Paths {
    fn new() -> Self {
        Self {
            reactive: q!(stardom_reactive),
            macros: q!(stardom_macros),
            named: q!(stardom_core::named),
            bindings: q!(stardom_core::__macro::bindings),
            web_sys: q!(stardom_core::__macro::web_sys),
            Node: q!(stardom_core::Node),
            IntoNode: q!(stardom_core::IntoNode),
            NodeRef: q!(stardom_core::NodeRef),
            IntoAttr: q!(stardom_core::attrs::IntoAttr),
            EventKey: q!(stardom_core::events::EventKey),
            EventOptions: q!(stardom_core::events::EventOptions),
        }
    }
}

pub fn paths() -> Paths {
    Paths::new()
}
