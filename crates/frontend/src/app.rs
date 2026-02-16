//! Main App component

use leptos::*;
use webshell_shared::TerminalSession;

use crate::terminal::Terminal;

/// Main application component - minimal full-screen terminal
#[component]
pub fn App() -> impl IntoView {
    let (session_id, set_session_id) = create_signal(Option::<String>::None);

    // Create single session on mount
    create_effect(move |_| {
        if session_id.get().is_none() {
            let session = TerminalSession::new(1);
            set_session_id.set(Some(session.id));
        }
    });

    view! {
        <div class="terminal-fullscreen">
            {move || {
                session_id.get().map(|id| {
                    view! { <Terminal session_id=id /> }
                })
            }}
        </div>
    }
}
