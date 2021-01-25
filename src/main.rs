mod protector;
use protector::{get_session_key, get_hash_key, SessionProtector};
use std::env;
use std::net::{SocketAddr, TcpStream, TcpListener};
use std::str::{FromStr, from_utf8};
use std::io;
use std::io::{Write, Read};
use std::thread::spawn;

fn main() {
    let args = env::args().collect::<Vec<_>>();

    let mut n = 1;
    if let Some(x) = args.iter().enumerate().find(|x| x.1 == "-n") {
        n = args[x.0 + 1].parse().unwrap();
    }

    if let Ok(addr) =  SocketAddr::from_str(args[1].as_str()) {
        client(addr);
    } else {
        let addr = "127.0.0.1:".to_string() + args[1].as_str();
        let addr = SocketAddr::from_str(addr.as_str()).unwrap();
        server(addr, n)
    }
}

fn client(addr: SocketAddr) {
    let mut stream = TcpStream::connect(addr).unwrap();

    println!("CONNECTED TO {}", addr);

    let hash = get_hash_key();
    println!("HASH: {}", hash);
    stream.write(&hash.as_bytes()).unwrap();

    let protector = SessionProtector::new(hash);

    let mut session_key = get_session_key();
    loop {
        println!("SESSION KEY: {}", session_key);
        stream.write(&session_key.as_bytes()).unwrap();

        print!("Message:");
        io::stdout().flush().unwrap();
        let mut message = String::new();

        io::stdin().read_line(&mut message);
        let mut message = message.into_bytes();
        message.resize(64, b' ');

        stream.write(&message).unwrap();


        let mut buf = [0u8; 10];
        stream.read_exact(&mut buf).unwrap();
        let server_session_key = from_utf8(&buf).unwrap().to_string();

        session_key = protector.next_session_key(session_key);
        if session_key == server_session_key {
            println!("SERVER SESSION KEY: \"{}\"", server_session_key);
        } else { panic!("WRONG SESSION KEY");}
    }
}

fn server(addr: SocketAddr, n: usize) {
    let listener = TcpListener::bind(addr).unwrap();
    println!("SERVER STARTED ON {}", addr);
    let mut count = 0;
    for mut stream in listener.incoming() {
        let stream = stream.unwrap();
        if count >= n {
            println!("SERVER IS FULL");
            continue;
        }
        count += 1;
        println!("{} CONNECTED", stream.peer_addr().unwrap());
        spawn(move || {
            let mut stream= stream;

            let mut buf = [0u8; 5];

            stream.read_exact(&mut buf).unwrap();

            let hash = from_utf8(&buf).unwrap();
            println!("HASH FROM {}: \"{}\"", stream.peer_addr().unwrap(), hash);
            let protector = SessionProtector::new(hash.to_string());

            loop {
                let mut buf = [0u8; 10];
                stream.read_exact(&mut buf).unwrap();
                let mut session_key = from_utf8(&buf).unwrap().to_string();
                println!("SESSION KEY FROM {}: \"{}\"", stream.peer_addr().unwrap(), session_key);

                let mut buf = [0u8; 64];
                stream.read_exact(&mut buf).unwrap();
                let message = from_utf8(&buf).unwrap().to_string();
                let message = message.trim();

                println!("FROM {}: \"{}\"", stream.peer_addr().unwrap(), message);

                session_key = protector.next_session_key(session_key);
                println!("NEXT SESSION KEY FOr {}: \"{}\"", stream.peer_addr().unwrap(), session_key);
                stream.write(&session_key.as_bytes());
            }
        });

    }
}
