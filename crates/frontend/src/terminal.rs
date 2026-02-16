//! Terminal component using xterm.js

use std::cell::RefCell;
use std::rc::Rc;

use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use webshell_shared::{ShellOutput, TerminalInput, TerminalOpenRequest, TerminalResize, WsMessage};

use crate::ws::WsClient;
use crate::xterm::{FitAddon, Terminal as XTerm, TerminalOptions};

/// Terminal component that manages xterm.js and WebSocket
#[component]
pub fn Terminal(session_id: String) -> impl IntoView {
    let container_ref = create_node_ref::<leptos::html::Div>();
    let session_id_clone = session_id.clone();

    // Set up the terminal on mount
    create_effect(move |_| {
        let session_id = session_id_clone.clone();

        // Get the container element
        if let Some(container) = container_ref.get() {
            let html_el: &web_sys::HtmlElement = container.as_ref();
            let container: HtmlElement = html_el.clone();

            // Create xterm.js terminal
            let options = TerminalOptions::default();
            let xterm = XTerm::new(&options.to_js_value());

            // Create and load fit addon
            let fit_addon = FitAddon::new_fit_addon();
            xterm.load_addon(&fit_addon.unchecked_ref());

            // Open terminal in container
            xterm.open(&container);

            // Fit to container after a small delay
            let fit_addon_rc = Rc::new(RefCell::new(fit_addon));
            let fit_addon_initial = fit_addon_rc.clone();

            let window = web_sys::window().unwrap();

            let fit_closure = Closure::wrap(Box::new(move || {
                fit_addon_initial.borrow().fit();
            }) as Box<dyn FnMut()>);

            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                fit_closure.as_ref().unchecked_ref(),
                50,
            );
            fit_closure.forget();

            let cols = xterm.cols();
            let rows = xterm.rows();

            // Store xterm reference
            let xterm_rc = Rc::new(RefCell::new(xterm));

            // Set up WebSocket connection
            let xterm_for_ws = xterm_rc.clone();
            let session_id_for_ws = session_id.clone();

            let on_message = move |msg: WsMessage| match msg {
                WsMessage::ShellOutput(ShellOutput { id, output }) => {
                    if id == session_id_for_ws {
                        xterm_for_ws.borrow().write(&output);
                    }
                }
                WsMessage::ShellExit(exit) => {
                    if exit.id == session_id_for_ws {
                        let code = exit
                            .code
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        xterm_for_ws.borrow().write(&format!(
                            "\r\n\x1b[33m[Process exited with code {}]\x1b[0m\r\n",
                            code
                        ));
                    }
                }
                _ => {}
            };

            match WsClient::new(on_message) {
                Ok(client) => {
                    let client_rc = Rc::new(RefCell::new(client));

                    // Wait for connection, then send open request
                    let client_for_open = client_rc.clone();
                    let session_id_for_open = session_id.clone();
                    let open_closure = Closure::wrap(Box::new(move || {
                        let _ = client_for_open.borrow().send(&WsMessage::TerminalOpen(
                            TerminalOpenRequest {
                                id: session_id_for_open.clone(),
                                cols,
                                rows,
                            },
                        ));
                    }) as Box<dyn FnMut()>);

                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        open_closure.as_ref().unchecked_ref(),
                        100,
                    );
                    open_closure.forget();

                    // Handle terminal input
                    let client_for_input = client_rc.clone();
                    let session_id_for_input = session_id.clone();
                    let on_data = Closure::wrap(Box::new(move |data: String| {
                        let _ = client_for_input.borrow().send(&WsMessage::TerminalInput(
                            TerminalInput {
                                id: session_id_for_input.clone(),
                                input: data,
                            },
                        ));
                    }) as Box<dyn FnMut(String)>);

                    xterm_rc.borrow().on_data(&on_data);
                    on_data.forget();

                    // Handle resize with ResizeObserver
                    let client_for_resize = client_rc.clone();
                    let xterm_for_resize = xterm_rc.clone();
                    let fit_addon_for_resize = fit_addon_rc.clone();
                    let session_id_for_resize = session_id.clone();

                    let resize_callback = Closure::wrap(Box::new(
                        move |_entries: js_sys::Array, _observer: web_sys::ResizeObserver| {
                            fit_addon_for_resize.borrow().fit();
                            let cols = xterm_for_resize.borrow().cols();
                            let rows = xterm_for_resize.borrow().rows();
                            let _ = client_for_resize.borrow().send(&WsMessage::TerminalResize(
                                TerminalResize {
                                    id: session_id_for_resize.clone(),
                                    cols,
                                    rows,
                                },
                            ));
                        },
                    )
                        as Box<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>);

                    let resize_observer =
                        web_sys::ResizeObserver::new(resize_callback.as_ref().unchecked_ref())
                            .unwrap();
                    resize_observer.observe(&container);
                    resize_callback.forget();

                    // Focus terminal
                    xterm_rc.borrow().focus();
                }
                Err(e) => {
                    log::error!("Failed to create WebSocket: {:?}", e);
                    xterm_rc
                        .borrow()
                        .write("\x1b[31m[Failed to connect to server]\x1b[0m\r\n");
                }
            }
        }
    });

    view! {
        <div
            node_ref=container_ref
            class="terminal-content"
            style="height: 100%; width: 100%;"
        />
    }
}
