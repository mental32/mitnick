use std::path::PathBuf;

use futures::{SinkExt, StreamExt};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "May the command line live forever!")]
enum Mitnick {
    /// Used to initialize a new world.
    Init {
        #[structopt(long)]
        redis_address: Option<String>,

        #[structopt(long)]
        hosts: usize,

        #[structopt(long)]
        output: PathBuf,
    },

    /// Start running the core.
    Run {
        #[structopt(long)]
        redis_address: Option<String>,
    },
}

// #[paw::main]
// #[actix_rt::main]
#[tokio::main]
async fn main() {
    let args = Mitnick::from_args();

    match args {
        Mitnick::Init { .. } => {}
        Mitnick::Run { .. } => {
            let context = tmq::Context::new();
            let mut socket = tmq::router(&context)
                .bind("ipc://tmp/mitnick-core")
                .unwrap();

            use mitnick::net::NetworkEvent;

            while let Some(Ok(message)) = socket.next().await {
                if let Ok(event) = bincode::deserialize::<NetworkEvent>(&message[1]) {
                    match event {
                        NetworkEvent::Heartbeat { .. }
                        | NetworkEvent::Suspend { .. }
                        | NetworkEvent::Resume { .. } => {}

                        NetworkEvent::Connect { ident, address } => {
                            println!("Connection from {:?} => ident={:?}", address, ident);
                        }

                        NetworkEvent::Data { ident, body } => {
                            if let Ok(text) = String::from_utf8(body) {
                                let data = bincode::serialize(&NetworkEvent::Data {
                                    ident,
                                    body: text.bytes().collect::<Vec<_>>(),
                                })
                                .unwrap();

                                let address = message[0].to_vec();

                                let response = vec![address, data];
                                let _ = socket.send(response).await;
                            }
                        }

                        NetworkEvent::Disconnect { ident: _ } => {
                            let _ = socket.send(message).await;
                        }
                    }
                }
            }
        }
    }
}
