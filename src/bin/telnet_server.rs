use std::collections::HashMap;

use mitnick::net::NetworkEvent;

use tokio::net::{TcpListener, TcpStream};

const IAC_DO_SA: &[u8] = [0xFF, 0xFD, 0x03];
const IAC_DO_LN: &[u8] = [0xFF, 0xFD, 0x22];
const IAC_WONT_ECHO: &[u8] = [0xFF, 0xFB, 0x01];

type SessionIdent = usize;

struct SessionState {
    address: (),
}

type SessionMap = HashMap<SessionIdent, SessionState>;

async fn telnet_prepare_session(socket: TcpStream) -> (_, (_, _)) {
    socket.send(&IAC_DO_SA).unwrap();
    socket.send(&IAC_DO_LN).unwrap();
    socket.send(&IAD_WONT_ECHO).unwrap();

    let address = socket.peer_address().unwrap();

    (address, socket.into_split())
}

#[tokio::main]
async fn main() {
    let mut tcp_listener = TcpListener::bind("127.0.0.1:8900").await.unwrap();

    let session_map = SessionMap::new();

    while let Ok(tcp_socket) = tcp_listener.accept().await {
        let identifier = session_map
            .keys()
            .max()
            .map(|i| i.wrapping_add(1))
            .unwrap_or(0);

        eprintln!("{:?}", (identifier, tcp_socket));

        let (address, (mut reader, mut writer)) = telnet_prepare_session(tcp_socket).await;

        // Reader task
        tokio::spawn(async move {
            let mut buf = [0u8];

            while let Ok(n) = reader.read_exact(&mut buf).await {
                if n == 0 {
                    break;
                }

                let event = NetworkEvent::Data {
                    ident,
                    body: buf.to_vec(),
                };

                let _ = data_tx.send(event);
            }

            let _ = data_tx.send(NetworkEvent::Disconnect { ident });
        });

        // Writer task
        tokio::spawn(async move {

        });
    }
}
