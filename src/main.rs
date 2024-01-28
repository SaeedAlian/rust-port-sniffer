use bpaf::Bpaf;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{channel, Sender};
use tokio::net::TcpStream;
use tokio::task;

const MAX_PORT: u16 = 4096;
const IP_FALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
pub struct Args {
    #[bpaf(long, short, argument("Address"), fallback(IP_FALLBACK))]
    /// The address that you want to sniff.  Must be a valid ipv4 address.  Falls back to 127.0.0.1
    pub address: IpAddr,

    #[bpaf(
        long("start"),
        short('s'),
        guard(start_port_guard, "Must be greater than 0"),
        fallback(1u16)
    )]
    /// The start port for the sniffer. (must be greater than 0)
    pub start_port: u16,

    #[bpaf(
        long("end"),
        short('e'),
        guard(end_port_guard, "Must be less than or equal to 4096"),
        fallback(MAX_PORT)
    )]
    /// The end port for the sniffer. (must be less than or equal to 4096)
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input <= MAX_PORT
}

async fn scan(sender: Sender<u16>, port: u16, ipaddr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", ipaddr, port)).await {
        Ok(_) => {
            sender.send(port).unwrap();
        }
        Err(_) => {}
    }
}

#[tokio::main]
async fn main() {
    let opts = args().run();
    let (sender, receiver) = channel();

    println!("Locating for open ports...");

    for i in opts.start_port..opts.end_port {
        let sender = sender.clone();

        task::spawn(async move { scan(sender, i, opts.address).await });
    }

    let mut output = vec![];

    drop(sender);

    for p in receiver {
        output.push(p);
    }

    println!("");
    output.sort();

    for p in output {
        println!("{} is open", p);
    }
}
