//! Network primitives used by the core and any connection transports.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkEvent {
    Connect { ident: usize, address: SocketAddr },
    Disconnect { ident: usize },
    Data { ident: usize, body: Vec<u8> },
}
