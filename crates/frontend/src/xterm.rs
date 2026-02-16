//! xterm.js bindings for WebAssembly
//!
//! This module provides Rust bindings to xterm.js via wasm-bindgen.

use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

#[wasm_bindgen]
extern "C" {
    /// xterm.js Terminal class
    #[wasm_bindgen(js_namespace = ["window"], js_name = "XTerminal")]
    pub type Terminal;

    #[wasm_bindgen(constructor, js_namespace = ["window"], js_class = "XTerminal")]
    pub fn new(options: &JsValue) -> Terminal;

    #[wasm_bindgen(method)]
    pub fn open(this: &Terminal, container: &HtmlElement);

    #[wasm_bindgen(method)]
    pub fn write(this: &Terminal, data: &str);

    #[wasm_bindgen(method)]
    pub fn focus(this: &Terminal);

    #[wasm_bindgen(method)]
    pub fn dispose(this: &Terminal);

    #[wasm_bindgen(method, getter)]
    pub fn cols(this: &Terminal) -> u16;

    #[wasm_bindgen(method, getter)]
    pub fn rows(this: &Terminal) -> u16;

    #[wasm_bindgen(method, js_name = "onData")]
    pub fn on_data(this: &Terminal, callback: &Closure<dyn FnMut(String)>);

    #[wasm_bindgen(method, js_name = "loadAddon")]
    pub fn load_addon(this: &Terminal, addon: &JsValue);

    /// xterm.js FitAddon class
    #[wasm_bindgen(js_namespace = ["window"], js_name = "XFitAddon")]
    pub type FitAddon;

    #[wasm_bindgen(constructor, js_namespace = ["window"], js_class = "XFitAddon")]
    pub fn new_fit_addon() -> FitAddon;

    #[wasm_bindgen(method)]
    pub fn fit(this: &FitAddon);

    /// Convert FitAddon to JsValue for loadAddon
    #[wasm_bindgen(method, js_name = "valueOf")]
    pub fn as_js_value(this: &FitAddon) -> JsValue;
}

/// Terminal options structure
pub struct TerminalOptions {
    pub cursor_blink: bool,
    pub cursor_style: String,
    pub font_size: u16,
    pub font_family: String,
    pub scrollback: u32,
}

impl TerminalOptions {
    pub fn to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();

        js_sys::Reflect::set(&obj, &"cursorBlink".into(), &self.cursor_blink.into()).unwrap();
        js_sys::Reflect::set(
            &obj,
            &"cursorStyle".into(),
            &self.cursor_style.clone().into(),
        )
        .unwrap();
        js_sys::Reflect::set(&obj, &"fontSize".into(), &self.font_size.into()).unwrap();
        js_sys::Reflect::set(&obj, &"fontFamily".into(), &self.font_family.clone().into()).unwrap();
        js_sys::Reflect::set(&obj, &"scrollback".into(), &self.scrollback.into()).unwrap();
        js_sys::Reflect::set(&obj, &"allowTransparency".into(), &true.into()).unwrap();
        js_sys::Reflect::set(&obj, &"convertEol".into(), &true.into()).unwrap();

        // Theme
        let theme = js_sys::Object::new();
        js_sys::Reflect::set(&theme, &"background".into(), &"#1a1b26".into()).unwrap();
        js_sys::Reflect::set(&theme, &"foreground".into(), &"#d4d4d4".into()).unwrap();
        js_sys::Reflect::set(&theme, &"cursor".into(), &"#aeafad".into()).unwrap();
        js_sys::Reflect::set(&theme, &"cursorAccent".into(), &"#000000".into()).unwrap();
        js_sys::Reflect::set(&theme, &"selectionBackground".into(), &"#264f78".into()).unwrap();
        js_sys::Reflect::set(&theme, &"black".into(), &"#000000".into()).unwrap();
        js_sys::Reflect::set(&theme, &"red".into(), &"#cd3131".into()).unwrap();
        js_sys::Reflect::set(&theme, &"green".into(), &"#0dbc79".into()).unwrap();
        js_sys::Reflect::set(&theme, &"yellow".into(), &"#e5e510".into()).unwrap();
        js_sys::Reflect::set(&theme, &"blue".into(), &"#2472c8".into()).unwrap();
        js_sys::Reflect::set(&theme, &"magenta".into(), &"#bc3fbc".into()).unwrap();
        js_sys::Reflect::set(&theme, &"cyan".into(), &"#11a8cd".into()).unwrap();
        js_sys::Reflect::set(&theme, &"white".into(), &"#e5e5e5".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightBlack".into(), &"#666666".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightRed".into(), &"#f14c4c".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightGreen".into(), &"#23d18b".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightYellow".into(), &"#f5f543".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightBlue".into(), &"#3b8eea".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightMagenta".into(), &"#d670d6".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightCyan".into(), &"#29b8db".into()).unwrap();
        js_sys::Reflect::set(&theme, &"brightWhite".into(), &"#ffffff".into()).unwrap();

        js_sys::Reflect::set(&obj, &"theme".into(), &theme).unwrap();

        obj.into()
    }
}

impl Default for TerminalOptions {
    fn default() -> Self {
        Self {
            cursor_blink: true,
            cursor_style: "block".to_string(),
            font_size: 14,
            font_family: "'JetBrains Mono', 'Fira Code', Consolas, monospace".to_string(),
            scrollback: 10000,
        }
    }
}
