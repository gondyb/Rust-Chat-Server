use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use crate::client_handler::Client;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::channels::Channel;
use crate::postman::{PostmanMessage, send_message};

pub struct BroadcastMessage {
    pub content: String,
    pub sender: Client,
    pub channel: String,
    pub send_to_sender: bool
}

pub fn start_broadcaster_thread(
    broadcast_rx: Receiver<BroadcastMessage>,
    postman_tx: Sender<PostmanMessage>,
    channels: Arc<Mutex<HashMap<String, Channel>>>) {
    thread::spawn(move || {
        loop {
            let msg = match broadcast_rx.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    println!("Error when receiving message {:?}", e);
                    continue
                }
            };

            let channels = match channels.lock() {
                Ok(channels) => channels,
                Err(e) => {
                    println!("Error when acquiring channels: {:?}", e);
                    continue
                }
            };

            let channel = match channels.get(&*msg.channel.clone()) {
                Some(channel) => channel,
                _ => {
                    println!("Channel {} doesn't exist", msg.channel.clone());
                    continue
                }
            };

            for client in channel.clients.iter() {
                if *client == msg.sender && msg.send_to_sender == false {
                    continue;
                }

                let postman_message = PostmanMessage {
                    client: client.clone(),
                    content: msg.content.clone()
                };

                send_message(postman_message, postman_tx.clone());
            }
        }
    });
}

pub fn send_broadcast_message(message: BroadcastMessage, broadcast_tx: Sender<BroadcastMessage>) {
    match broadcast_tx.send(message) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send message to broadcast channel: {:?}", e);
            return
        }
    }
}