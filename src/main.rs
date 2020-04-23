#![feature(option_expect_none)]

use std::collections::{HashMap, HashSet};
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
                .bind("ipc:///tmp/mitnick-core")
                .unwrap();

            use mitnick::net::{AccessToken, NetworkEvent};

            type Session = (usize, Vec<u8>);

            // [ZMQ_ADDRESS] -> Vec<identities>
            let mut connectors: HashMap<Vec<u8>, HashSet<usize>> = HashMap::new();

            // (identity, [ZMQ_ADDRESS]) -> Vec<Output>
            let mut sessions: HashMap<Session, Vec<Vec<u8>>> = HashMap::new();

            // <ACCESS_TOKEN> -> (identity, [ZMQ_ADDRESS])
            let mut access_map: HashMap<AccessToken, Session> = HashMap::new();

            while let Some(Ok(message)) = socket.next().await {
                let zmq_address = message[0].to_vec().clone();

                if let Ok(event) = bincode::deserialize::<NetworkEvent>(&message[1]) {
                    match event {
                        NetworkEvent::Data { ident, body }
                            if connectors
                                .get(&zmq_address)
                                .map(|s| s.contains(&ident))
                                .unwrap_or(false) =>
                        {
                            if let Some(stdout_streams) = sessions.get(&(ident, zmq_address)) {
                                if let Ok(text) = String::from_utf8(body) {
                                    let data = bincode::serialize(&NetworkEvent::Data {
                                        ident,
                                        body: text.bytes().collect::<Vec<_>>(),
                                    })
                                    .unwrap();

                                    for zmq_address in stdout_streams.iter().cloned() {
                                        let response = vec![zmq_address, data.clone()];
                                        let _ = socket.send(response).await;
                                    }
                                }
                            }
                        }

                        NetworkEvent::Connect { ident, address } => {
                            println!("Connection from {:?} => ident={:?}", address, ident);

                            connectors
                                .entry(zmq_address.clone())
                                .or_default()
                                .insert(ident);
                            sessions.insert((ident, zmq_address.clone()), vec![]);

                            let mut access_token: AccessToken = rand::random();

                            while access_map.contains_key(&access_token) {
                                access_token = rand::random();
                            }

                            access_map
                                .insert(access_token, (ident, zmq_address.clone()))
                                .expect_none("Collision!");

                            let resume = bincode::serialize(&NetworkEvent::Resume {
                                ident,
                                access_token: Some(access_token),
                            })
                            .unwrap();

                            let _ = socket.send(vec![zmq_address, resume]).await;
                        }

                        NetworkEvent::Disconnect { ident: _ } => {
                            let _ = socket.send(message).await;
                        }

                        NetworkEvent::Resume {
                            ident: _,
                            access_token,
                        } => {
                            if let Some(bucket) = access_token
                                .as_ref()
                                .and_then(|t| access_map.get(t))
                                .and_then(|k| sessions.get_mut(k))
                            {
                                bucket.push(zmq_address);
                            }
                        }

                        NetworkEvent::Heartbeat { .. }
                        | NetworkEvent::Suspend { .. }
                        | NetworkEvent::Data { .. } => {}
                    }
                }
            }
        }
    }
}
