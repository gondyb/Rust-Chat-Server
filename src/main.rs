mod messages;
mod client_handler;
mod broadcaster;

use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc, RwLock};
use std::net::{TcpListener, TcpStream};
use crate::broadcaster::start_broadcaster;
use crate::client_handler::handle_client;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

    let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();

    // Vector of currently connected clients
    let clients_vec: Vec<TcpStream> = Vec::new();

    // let clients_lock: RwLock<Vec<TcpStream>> = RwLock::new(clients_vec.clone());
    let clients_arc: Arc<RwLock<Vec<TcpStream>>> = Arc::new(RwLock::new(clients_vec));

    // let clients_lock_clone = clients_lock.clone();
    let clients_arc_clone = clients_arc.clone();

    start_broadcaster(clients_arc_clone, receiver);

    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => { // New client connected
                // Thread to handle the client
                clients_arc.write().unwrap().push(stream.try_clone().expect("Cannot clone stream"));
                let sender = sender.clone();
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream, sender);
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}