use std::env;
use std::net::{IpAddr, TcpStream};
use std::process;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

const MAX: u16 = 2048;

struct Args {
    ipaddr: IpAddr,
    threads: u16,
}

impl Args {
    fn new(args: &Vec<String>) -> Result<Args, &'static str> {
        let args_len = args.len();

        if args_len > 4 {
            return Err("Too many arguments");
        } else if args_len < 2 {
            return Err("Not enough arguments");
        }

        let flag = args[1].clone();

        if let Ok(ipaddr) = IpAddr::from_str(&flag) {
            return Ok(Args { threads: 5, ipaddr });
        } else {
            if flag.contains("-h") || flag.contains("--help") && args_len == 2 {
                println!(
                    "Usage:
                        \r-j to select how many threads you want
                        \r-h or --help to show this help message"
                );
                return Err("Help");
            } else if flag.contains("-h") || flag.contains("--help") {
                return Err("Too many arguments");
            } else if flag.contains("-j") {
                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(ip) => ip,
                    Err(_) => return Err("IP address is not valid"),
                };

                let threads = match args[2].parse::<u16>() {
                    Ok(t) => t,
                    Err(_) => return Err("Threads must be number"),
                };

                return Ok(Args { threads, ipaddr });
            } else {
                return Err("Invalid arguments");
            }
        }
    }
}

fn scan(sender: Sender<u16>, start_port: u16, ipaddr: IpAddr, threads: u16) {
    let mut port: u16 = start_port + 1;

    loop {
        match TcpStream::connect((ipaddr, port)) {
            Ok(_) => {
                sender.send(port).unwrap();
            }
            Err(_) => {}
        }

        if (MAX - port) <= threads {
            break;
        }

        port += threads;
    }
}

fn main() {
    let args_collection: Vec<String> = env::args().collect();

    let args = Args::new(&args_collection).unwrap_or_else(|err| {
        if err.contains("Help") {
            process::exit(0);
        } else {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    });

    let threads = args.threads;
    let ipaddr = args.ipaddr;
    let (sender, receiver) = channel();

    println!("Locating for open ports...");

    for i in 0..threads {
        let sender = sender.clone();

        thread::spawn(move || {
            scan(sender, i, ipaddr, threads);
        });
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
