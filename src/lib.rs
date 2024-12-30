use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Event {
    Chat(String, String),
    Connected(String),
    Disconnected(String),
}
