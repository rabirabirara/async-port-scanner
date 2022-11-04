use clap::Parser;
use std::io::{self, Write};
use async_std::channel;
use async_std::net::{IpAddr, TcpStream};
use async_std::task::spawn;
use std::str::FromStr;


const MAX_PORT: u16 = 65535;


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg()]
    ipaddr: String,
}

async fn scan(tx: channel::Sender<u16>, port: u16, addr: IpAddr) {
    match TcpStream::connect((addr, port)).await {
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(port).await.unwrap();
        }
        Err(_) => (),
    }
}

#[async_std::main]
async fn main() {
    let args = Args::parse();
    if let Ok(ipaddr) = IpAddr::from_str(&args.ipaddr) {
        let (tx, rx) = channel::unbounded::<u16>();

        let mut handles = Vec::new();

        for i in 1..=MAX_PORT {
            let txc = tx.clone();
            handles.push(spawn(async move {
                scan(txc, i, ipaddr).await;
            }));
        }

        drop(tx);

        // We wait for all the spawned tasks to complete.  If not, the program will continue to
        // run and exit.
        for h in handles {
            h.await;
        }

        let mut ports = Vec::new();

        // receive all last messages; this will error on closed+empty channel
        while let Ok(p) = rx.recv().await {
            ports.push(p);
        }

        ports.sort();
        println!("\n{ipaddr}:");
        for p in ports {
            println!("{p} is open");
        }
    } else {
        // print help, though CLAP does it automatically
    }
}
