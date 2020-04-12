//! Network primitives used by the core and any connection transports.

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkEvent {
    Connect { ident: usize },
    Disconnect { ident: usize },
    Data { ident: usize, body: Vec<u8> },
}
