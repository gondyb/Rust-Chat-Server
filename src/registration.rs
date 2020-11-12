use std::sync::mpsc::{Receiver, Sender};
use crate::client_handler::Client;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::postman::{PostmanMessage, send_message};
use crate::protocol::welcome_reply;
use crate::channels::{ChannelMessage};

pub enum RegistrationAction {
    REGISTER,
    LEAVE
}

pub struct RegistrationMessage {
    pub client: Client,
    pub action: RegistrationAction
}

// Registration registers and unregisters client from connected clients
pub fn start_registration_thread(
    clients: Arc<Mutex<Vec<Client>>>,
    registration_rx: Receiver<RegistrationMessage>,
    channels_tx: Sender<ChannelMessage>,
    postman_tx: Sender<PostmanMessage>
) {
    thread::spawn(move || {
        loop {
            // Listen to the channel
            let registration_message =  match registration_rx.recv() {
                Ok(client) => client,
                Err(e) => {
                    println!("Error when receiving from registration channel: {:?}", e);
                    continue
                }
            };

            match registration_message.action {
                // Registers a client
                RegistrationAction::REGISTER => {
                    register_client(
                        registration_message.client,
                        clients.clone(),
                        postman_tx.clone()
                    );
                },
                // Unregister a client
                RegistrationAction::LEAVE => {
                    unregister_client(registration_message.client, clients.clone(), channels_tx.clone());
                }
            }



        }
    });
}

fn register_client(client: Client, clients: Arc<Mutex<Vec<Client>>>, postman_tx: Sender<PostmanMessage>) {
    // Say Hello to new client
    let msg = PostmanMessage {
        client: client.clone(),
        content: welcome_reply(client.username.clone())
    };

    send_message(msg, postman_tx.clone());

    let mut clients = match clients.lock() {
        Ok(clients) => clients,
        Err(e) => {
            println!("Register: Unable to acquire clients lock: {:?}", e);
            return
        }
    };

    println!("New client registered: {:?}", client.username.clone());
    // Add new client to connected clients vector
    clients.push(client);
}

// Unregister a client if its connection broke or after QUIT message
fn unregister_client(client: Client, clients: Arc<Mutex<Vec<Client>>>, channels: Sender<ChannelMessage>) {
    let mut clients = match clients.lock() {
        Ok(clients) => clients,
        Err(e) => {
            println!("Unregister: Unable to acquire clients lock: {:?}", e);
            return
        }
    };

    // Remove client from clients vector
    clients.retain(|c| c.clone() != client);

    let channel_unregister = ChannelMessage {
       client,
       channel: None,
       body: None,
       leave: true,
       all_channels: true
    };

    // Tell channels to remove client to every channel
    match channels.send(channel_unregister) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send unregister to all chanels message: {:?}", e);
            return
        }
    }
}