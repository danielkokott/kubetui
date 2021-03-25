pub mod input;
pub mod tick;

pub mod kubernetes;

mod util;

use crossterm::event::KeyEvent;

pub enum Event {
    Input(KeyEvent),
    Kube(Kube),
    Tick,
    Resize(u16, u16),
    Mouse,
}

pub enum Kube {
    // Context
    CurrentContextRequest,
    CurrentContextResponse(String, String), // current_context, namespace
    // Namespace
    GetNamespacesRequest,
    GetNamespacesResponse(Vec<String>),
    SetNamespace(String),
    // Pod Logs
    Pod(Vec<String>),
    LogStreamRequest(String),
    LogStreamResponse(Vec<String>),
    // ConfigMap & Secret
    Configs(Vec<String>),
    ConfigRequest(String),
    ConfigResponse(Vec<String>),
}
