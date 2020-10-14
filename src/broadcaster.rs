use std::thread;
use std::sync::{Arc, RwLock};
use std::net::TcpStream;
use std::io::Write;
use std::sync::mpsc::Receiver;

pub fn start_broadcaster(clients: Arc<RwLock<Vec<TcpStream>>>, receiver: Receiver<String>) {
    thread::spawn(move || {
        loop {
            let msg = receiver.recv();
            match msg {
                Ok(m) => {
                    let clients = clients.read().unwrap();
                    for mut stream in clients.iter() {
                        stream.write(&m.as_bytes());
                        stream.flush();
                    }
                },
                Err(e) => {
                    println!("Error when receiving message {}", e);
                }
            }
        }
    });
}