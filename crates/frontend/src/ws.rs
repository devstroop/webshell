//! WebSocket client for terminal communication

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

use webshell_shared::WsMessage;

/// WebSocket connection state
pub struct WsClient {
    socket: WebSocket,
    #[allow(dead_code)]
    on_message_closure: Closure<dyn FnMut(MessageEvent)>,
    #[allow(dead_code)]
    on_open_closure: Closure<dyn FnMut()>,
    #[allow(dead_code)]
    on_close_closure: Closure<dyn FnMut(CloseEvent)>,
    #[allow(dead_code)]
    on_error_closure: Closure<dyn FnMut(ErrorEvent)>,
}

impl WsClient {
    /// Create a new WebSocket connection
    pub fn new<F>(on_message: F) -> Result<Self, JsValue>
    where
        F: Fn(WsMessage) + 'static,
    {
        let ws_url = get_ws_url();
        log::info!("Connecting to WebSocket: {}", ws_url);

        let socket = WebSocket::new(&ws_url)?;
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Message handler
        let on_message = Rc::new(RefCell::new(on_message));
        let on_message_clone = on_message.clone();
        let on_message_closure = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                    (on_message_clone.borrow())(msg);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        socket.set_onmessage(Some(on_message_closure.as_ref().unchecked_ref()));

        // Open handler
        let on_open_closure = Closure::wrap(Box::new(move || {
            log::info!("WebSocket connected");
        }) as Box<dyn FnMut()>);
        socket.set_onopen(Some(on_open_closure.as_ref().unchecked_ref()));

        // Close handler
        let on_close_closure = Closure::wrap(Box::new(move |e: CloseEvent| {
            log::info!("WebSocket closed: code={}, reason={}", e.code(), e.reason());
        }) as Box<dyn FnMut(CloseEvent)>);
        socket.set_onclose(Some(on_close_closure.as_ref().unchecked_ref()));

        // Error handler
        let on_error_closure = Closure::wrap(Box::new(move |e: ErrorEvent| {
            log::error!("WebSocket error: {:?}", e.message());
        }) as Box<dyn FnMut(ErrorEvent)>);
        socket.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));

        Ok(Self {
            socket,
            on_message_closure,
            on_open_closure,
            on_close_closure,
            on_error_closure,
        })
    }

    /// Send a message to the server
    pub fn send(&self, msg: &WsMessage) -> Result<(), JsValue> {
        if self.socket.ready_state() != WebSocket::OPEN {
            log::warn!("WebSocket not open, cannot send message");
            return Ok(());
        }

        let json = serde_json::to_string(msg).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.socket.send_with_str(&json)
    }

    /// Check if the WebSocket is connected
    pub fn is_connected(&self) -> bool {
        self.socket.ready_state() == WebSocket::OPEN
    }

    /// Close the WebSocket connection
    pub fn close(&self) {
        let _ = self.socket.close();
    }
}

impl Drop for WsClient {
    fn drop(&mut self) {
        self.close();
    }
}

/// Get the WebSocket URL based on current location (same origin)
fn get_ws_url() -> String {
    let window = web_sys::window().expect("no global window");
    let location = window.location();
    
    let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
    let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
    let host = location.host().unwrap_or_else(|_| "localhost:3000".to_string());
    
    format!("{}//{}/ws", ws_protocol, host)
}
