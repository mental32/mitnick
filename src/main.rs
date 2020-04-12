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

// #[paw::main]
#[actix_rt::main]
async fn main() {
    let args = Mitnick::from_args();

    match args {
        Mitnick::Init { hosts, output } => {},
        Mitnick::Run { redis_addr } => {
            let context = zmq::Context::new();
            let socket = context.socket(zmq::ROUTER).unwrap();

            let _ = socket.bind("ipc:///tmp/mitnick-core");

            let _ = thread::spawn(move || {
                while let Ok(message) = socket.recv_multipart(0x00) {
                    println!("Got message! {:?}", message);
                }

            }).join();
        },
    }
}
