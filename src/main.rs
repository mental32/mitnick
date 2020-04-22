use std::path::PathBuf;
use std::thread;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "May the command line live forever!")]
enum Mitnick {
    Init {
        #[structopt(long)]
        hosts: usize,

        #[structopt(long)]
        output: PathBuf,
    },

    Run {
        #[structopt(long)]
        redis_addr: Option<String>,
    },
}

#[paw::main]
// #[actix_rt::main]
fn main(args: Mitnick) {
    match args {
        Mitnick::Init { hosts, output } => {}
        Mitnick::Run { redis_addr } => {
            let context = zmq::Context::new();
            let socket = context.socket(zmq::ROUTER).unwrap();

            let _ = socket.bind("ipc:///tmp/mitnick-core");

            use mitnick::net::NetworkEvent;

            while let Ok(message) = socket.recv_multipart(0x00) {
                if let Ok(event) = bincode::deserialize::<NetworkEvent>(&message[1]) {
                    match event {
                        NetworkEvent::Heartbeat { .. } => {

                        },

                        NetworkEvent::Connect { ident, address } => {
                            println!("Connection from {:?} => ident={:?}", address, ident);
                        },

                        NetworkEvent::Data { ident, body } => {
                            if let Ok(text) = String::from_utf8(body) {
                                // println!("{:?} => {:?}", ident, text);

                                let data = bincode::serialize(&NetworkEvent::Data {
                                    ident,
                                    body: text.bytes().collect::<Vec<_>>(),
                                })
                                .unwrap();
    
                                let response = vec![message[0].clone(), data];
                                let _ = socket.send_multipart(response, 0x00);
                            }
                        },

                        NetworkEvent::Disconnect { ident: _ } => {
                            let _ = socket.send_multipart(message, 0x00);
                        }
                    }
                }
            }
        }
    }
}
