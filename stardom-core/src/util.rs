pub fn window() -> web_sys::Window {
    web_sys::window().expect("JavaScript window undefined")
}

pub fn document() -> web_sys::Document {
    window()
        .document()
        .expect("JavaScript window.document undefined")
}
