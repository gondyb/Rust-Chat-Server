use std::net::{TcpListener};
use std::sync::{mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::client_handler::{Client, handle_client};
use crate::server::{BroadcastMessage, ChannelMessage, start_server};

mod client_handler;
mod server;
mod protocol;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

    let (broadcast_tx, broadcast_rx): (Sender<BroadcastMessage>, Receiver<BroadcastMessage>) = mpsc::channel();
    let (registration_tx, registration_rx): (Sender<Client>, Receiver<Client>) = mpsc::channel();
    let (channel_tx, channel_rx): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();

    start_server(broadcast_tx.clone(), registration_rx, broadcast_rx, channel_rx);

    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3333");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => { // New client connected
                let btx = broadcast_tx.clone();
                let rtx = registration_tx.clone();
                let ctx = channel_tx.clone();
                thread::spawn(move || {
                    // connection succeeded
                    handle_client(stream, btx, rtx, ctx);
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