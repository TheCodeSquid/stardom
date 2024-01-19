use std::thread_local;

thread_local! {
    static DOCUMENT: Option<web_sys::Document> = if cfg!(target_family = "wasm") {
        web_sys::window()
            .and_then(|window| window.document())
    } else {
        None
    };
}

pub fn document() -> Option<web_sys::Document> {
    DOCUMENT.with(Clone::clone)
}

pub fn is_web() -> bool {
    document().is_some()
}
