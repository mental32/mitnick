//! Network primitives used by the core and any connection transports.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

pub type AccessToken = [u8; 8];

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkEvent {
    Connect {
        ident: usize,
        address: SocketAddr,
        access_token: AccessToken,
    },

    Disconnect {
        ident: usize,
    },

    Data {
        ident: usize,
        body: Vec<u8>,
    },

    Heartbeat {
        sequence: usize,
    },

    Suspend {
        ident: usize,
        message: Option<String>,
    },

    Resume {
        ident: usize,
    },
}

impl From<NetworkEvent> for tmq::Multipart {
    fn from(ev: NetworkEvent) -> Self {
        let s = bincode::serialize(&ev).unwrap();
        let m: tmq::Message = s.into();
        m.into()
    }
}
