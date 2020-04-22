use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;

use mitnick::net::NetworkEvent;

use crossbeam_channel;

use tokio::prelude::*;
// use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[tokio::main]
async fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::DEALER).unwrap();
    socket.connect("ipc:///tmp/mitnick-core").unwrap();

    let local_rx = context.socket(zmq::PULL).unwrap();
    local_rx.bind("inproc://broker").unwrap();

    let (data_tx, data_rx) = crossbeam_channel::bounded(0);

    let mut listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    thread::spawn(move || {
        let local_tx = context.socket(zmq::PUSH).unwrap();
        local_tx.connect("inproc://broker").unwrap();
        for event in data_rx {
            let _ = local_tx.send(bincode::serialize(&event).unwrap(), 0x00);
        }
    });

    let mut sessions: Arc<Mutex<HashMap<usize, UnboundedSender<NetworkEvent>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    {
        let sessions = sessions.clone();

        thread::spawn(move || {
            let mut items = vec![
                socket.as_poll_item(zmq::PollEvents::POLLIN),
                local_rx.as_poll_item(zmq::PollEvents::POLLIN),
            ];

            loop {
                if let Ok(_) = zmq::poll(&mut items, -1) {
                    if let Ok(outbound) = local_rx.recv_bytes(zmq::DONTWAIT) {
                        let _ = socket.send(outbound, 0x00);
                    }

                    if let Ok(inbound) = socket.recv_bytes(zmq::DONTWAIT) {
                        if let Ok(event) = bincode::deserialize::<NetworkEvent>(&inbound) {
                            let ident = match event {
                                NetworkEvent::Data { ident, body: _ } => ident,
                                NetworkEvent::Connect { ident } => ident,
                                NetworkEvent::Disconnect { ident } => ident,
                            };

                            println!("? {:?}", event);

                            dbg!(sessions.lock().unwrap().get(&ident))
                                .unwrap()
                                .send(event)
                                .unwrap();
                        }
                    }
                }
            }
        });
    }

    use std::sync::Arc;

    loop {
        let (mut reader, mut writer) = {
            let (socket, _) = listener.accept().await.unwrap();
            socket.into_split()
        };

        let (client_tx, mut client_rx) = unbounded_channel();

        let ident = {
            let mut sessions = sessions.lock().unwrap();
            let ident = sessions.keys().max().map_or(0, |n| n.wrapping_add(1));

            sessions.insert(ident, client_tx);
            ident
        };

        let data_tx = data_tx.clone();

        let _ = data_tx.send(NetworkEvent::Connect { ident });

        tokio::task::spawn(async move {
            while let Some(event) = client_rx.recv().await {
                match event {
                    NetworkEvent::Data { ident: _, body } => {
                        writer.write(body.as_slice()).await.unwrap();
                    }

                    _ => break,
                }
            }
        });

        tokio::task::spawn(async move {
            let mut buf = [0; 1];

            loop {
                while let Ok(n) = reader.read_exact(&mut buf).await {
                    if n == 0 {
                        let disconnect = NetworkEvent::Disconnect { ident };
                        let _ = data_tx.send(disconnect);
                        break;
                    } else {
                        let data = NetworkEvent::Data {
                            ident,
                            body: buf.to_vec(),
                        };

                        let _ = data_tx.send(data);
                    }
                }
            }
        });
    }
}
