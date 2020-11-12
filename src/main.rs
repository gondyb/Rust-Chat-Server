use std::net::{TcpListener};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

use crate::client_handler::{Client, start_client_thread};
use crate::channels::{ChannelMessage, start_channels_thread, Channel};
use crate::postman::{PostmanMessage, start_postman_thread};
use std::collections::HashMap;
use crate::registration::{start_registration_thread, RegistrationMessage};
use crate::broadcast::{start_broadcaster_thread, BroadcastMessage};

mod client_handler;
mod protocol;
mod channels;
mod postman;
mod registration;
mod broadcast;

fn main() {
    let listener = match TcpListener::bind("0.0.0.0:3333") {
        Ok(listener) => listener,
        Err(e) => {
            println!("Unable to bind port to socket: {:?}", e);
            return
        }
    };

    // Channels creation for each thread

    // Broadcast: Sends a message to all clients connected in a given channel
    let (broadcast_tx, broadcast_rx): (Sender<BroadcastMessage>, Receiver<BroadcastMessage>) = mpsc::channel();

    // Registration: Registers a client and adds it to the clients vector, or remove it
    let (registration_tx, registration_rx): (Sender<RegistrationMessage>, Receiver<RegistrationMessage>) = mpsc::channel();

    // Channel: Handle join and leave channel messages
    let (channel_tx, channel_rx): (Sender<ChannelMessage>, Receiver<ChannelMessage>) = mpsc::channel();

    // Postman: Sends a message to a client
    let (postman_tx, postman_rx): (Sender<PostmanMessage>, Receiver<PostmanMessage>) = mpsc::channel();

    start_postman_thread(postman_rx, registration_tx.clone());

    // Vector containing every connected clients
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    // HashMap containing channels, accessible by channel name
    let channels: Arc<Mutex<HashMap<String, Channel>>> = Arc::new(Mutex::new(HashMap::new()));

    start_channels_thread(
        channels.clone(),
        channel_rx,
        broadcast_tx.clone(),
        postman_tx.clone()
    );

    start_registration_thread(
        clients.clone(),
        registration_rx,
        channel_tx.clone(),
        postman_tx.clone()
    );

    start_broadcaster_thread(
        broadcast_rx,
        postman_tx,
        channels.clone()
    );

    println!("Server listening on port 3333");

    // Accept connection for each new client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => { // New client connected
                let btx = broadcast_tx.clone();
                let rtx = registration_tx.clone();
                let ctx = channel_tx.clone();

                start_client_thread(
                    stream, btx, rtx, ctx);
            }
            Err(e) => {
                println!("Error when accepting new client: {}", e);
            }
        }
    }
    // Close the socket server
    drop(listener);
}