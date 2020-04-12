use std::thread;
use std::collections::HashMap;

use mitnick::net::NetworkEvent;

use crossbeam_channel;

use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio::net::TcpListener;


#[tokio::main]
async fn main() {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::DEALER).unwrap();
    socket.connect("ipc:///tmp/mitnick-core").unwrap();

    let (data_tx, data_rx) = crossbeam_channel::bounded(0);

    let mut listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    thread::spawn(move || while let Ok(event) = data_rx.recv() {
        socket.send(bincode::serialize(&event).unwrap(), 0x00).unwrap()
    });

    let mut sessions: HashMap<usize, _> = HashMap::new();

    use std::sync::Arc;

    loop {
        let (cl, _) = listener.accept().await.unwrap();
        let cl = Arc::new(Mutex::new(cl));

        let ident = sessions.keys().max().map_or(0, |n| n.wrapping_add(1));

        sessions.insert(ident, cl.clone());

        let data_tx = data_tx.clone();

        let _ = data_tx.send(NetworkEvent::Connect { ident });

        tokio::spawn(async move {
            let mut buf = [0; 1];

            loop {
                let mut guard = cl.lock().await;

                if let Ok(n) = guard.read_exact(&mut buf).await {
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
