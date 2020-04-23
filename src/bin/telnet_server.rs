#![feature(type_ascription)]

use std::collections::HashMap;
use std::net::SocketAddr;

use mitnick::net::{AccessToken, NetworkEvent};

use futures::{SinkExt, StreamExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
};
use tokio::prelude::*;
use tokio::sync::mpsc;

const IAC_DO_SA: &[u8] = &[0xFF, 0xFD, 0x03];
const IAC_DO_LN: &[u8] = &[0xFF, 0xFD, 0x22];
const IAC_WONT_ECHO: &[u8] = &[0xFF, 0xFB, 0x01];

type SessionIdent = usize;

struct SessionState {
    // address: (),
}

type SessionMap = HashMap<SessionIdent, SessionState>;

async fn telnet_prepare_session(
    mut socket: TcpStream,
) -> (SocketAddr, (OwnedReadHalf, OwnedWriteHalf)) {
    socket.write(&IAC_DO_SA).await.unwrap();
    socket.write(&IAC_DO_LN).await.unwrap();
    socket.write(&IAC_WONT_ECHO).await.unwrap();

    let address = socket.peer_addr().unwrap();

    (address, socket.into_split())
}

#[tokio::main]
async fn main() {
    let mut tcp_listener = TcpListener::bind("127.0.0.1:8900").await.unwrap();

    let session_map = SessionMap::new();

    let context = tmq::Context::new();

    while let Ok((tcp_socket, _)) = tcp_listener.accept().await {
        let ident = session_map
            .keys()
            .max()
            .map(|i| i.wrapping_add(1))
            .unwrap_or(0);

        eprintln!("{:?}", (ident, &tcp_socket));

        let (mut access_tx, mut access_rx) = mpsc::channel::<AccessToken>(1);
        let (address, (mut reader, mut writer)) = telnet_prepare_session(tcp_socket).await;
        let (mut data_tx, mut data_rx) = (
            tmq::dealer(&context).connect("ipc:///tmp/mitnick-core").unwrap(),
            tmq::dealer(&context).connect("ipc:///tmp/mitnick-core").unwrap(),
        );

        data_tx
            .send(NetworkEvent::Connect { ident, address })
            .await
            .unwrap();

        // Reader task
        tokio::spawn(async move {
            match data_tx
                .next()
                .await
                .and_then(|r| r.map(|mut f| f.pop_front()).ok())
                .flatten()
                .map(|m| bincode::deserialize::<NetworkEvent>(&*m))
            {
                Some(Ok(NetworkEvent::Resume {
                    access_token: Some(access_token),
                    ..
                })) => access_tx.send(access_token).await.unwrap(),

                _ => {
                    std::mem::drop(access_tx);
                    return;
                }
            }

            let mut buf = [0u8];

            while let Ok(n) = reader.read_exact(&mut buf).await {
                if n == 0 {
                    break;
                }

                let event = NetworkEvent::Data {
                    ident,
                    body: buf.to_vec(),
                };

                let _ = data_tx.send(event).await;
            }

            let _ = data_tx.send(NetworkEvent::Disconnect { ident });
        });

        // Writer task
        tokio::spawn(async move {
            if let Some(access_token) = access_rx.recv().await {
                data_rx
                    .send(NetworkEvent::Resume {
                        ident,
                        access_token: Some(access_token),
                    })
                    .await
                    .unwrap();

                while let Some(event) = data_rx
                    .next()
                    .await
                    .and_then(|r| r.map(|mut f| f.pop_front()).ok())
                    .flatten()
                    .and_then(|m| bincode::deserialize::<NetworkEvent>(&*m).ok())
                {
                    match event {
                        NetworkEvent::Data { ident: _, body} => {
                            writer.write(&body).await.unwrap();
                        },

                        NetworkEvent::Disconnect { .. } => break,

                        _ => {},
                    }
                }
            }
        });
    }
}
