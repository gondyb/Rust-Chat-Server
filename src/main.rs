use std::net::{TcpListener};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::client_handler::{Client, handle_client};
use crate::channels::{ChannelMessage, start_channels_thread, Channel};
use crate::postman::{PostmanMessage, start_postman_thread};
use std::collections::HashMap;
use crate::registration::start_registration_thread;
use crate::broadcast::{start_broadcaster_thread, BroadcastMessage};

mod client_handler;
mod protocol;
mod channels;
mod postman;
mod registration;
mod broadcast;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

    let (broadcast_tx, broadcast_rx): (Sender<BroadcastMessage>, Receiver<BroadcastMessage>) = mpsc::channel();
    let (registration_tx, registration_rx): (Sender<Client>, Receiver<Client>) = mpsc::channel();
    let (channel_tx, channel_rx): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();
    let (postman_tx, postman_rx): (Sender<PostmanMessage>, Receiver<PostmanMessage>) = mpsc::channel();

    start_postman_thread(postman_rx);

    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    let channels: Arc<Mutex<HashMap<String, Channel>>> = Arc::new(Mutex::new(HashMap::new()));

    start_channels_thread(
        channels.clone(),
        channel_rx,
        broadcast_tx.clone()
    );

    start_registration_thread(
        clients.clone(),
        registration_rx,
        postman_tx.clone()
    );

    start_broadcaster_thread(
        broadcast_rx,
        postman_tx,
        channels.clone()
    );

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
                    handle_client(
                        stream, btx, rtx, ctx);
                });
            }
            Err(e) => {
                println!("Error when accepting new client: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}