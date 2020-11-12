use std::sync::mpsc::{Receiver, Sender};
use crate::client_handler::Client;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::postman::{PostmanMessage, send_message};
use crate::protocol::welcome_reply;

pub fn start_registration_thread(
    clients: Arc<Mutex<Vec<Client>>>,
    registration_rx: Receiver<Client>,
    postman_tx: Sender<PostmanMessage>
) {
    thread::spawn(move || {
        loop {
            let new_client =  match registration_rx.recv() {
                Ok(client) => client,
                Err(e) => {
                    println!("Error when receiving from registration channel: {:?}", e);
                    continue
                }
            };

            let msg = PostmanMessage {
                client: new_client.clone(),
                content: welcome_reply(new_client.username.clone())
            };

            send_message(msg, postman_tx.clone());

            let mut clients = match clients.lock() {
                Ok(clients) => clients,
                Err(e) => {
                    println!("Receiver: Unable to acquire clients lock: {:?}", e);
                    continue
                }
            };

            println!("New client registered: {:?}", new_client.username.clone());
            clients.push(new_client);

        }
    });
}