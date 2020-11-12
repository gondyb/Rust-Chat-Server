use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::io::{BufWriter, Write, ErrorKind};
use crate::client_handler::Client;
use crate::registration::{RegistrationMessage, RegistrationAction};

pub struct PostmanMessage {
    pub(crate) client: Client,
    pub(crate) content: String
}

// Postman sends a message to a client, but ASYNCHRONOUSLY
pub fn start_postman_thread(postman_rx: Receiver<PostmanMessage>, register_tx: Sender<RegistrationMessage>) {
    thread::spawn(move || {
        loop {
            // Read from channel
            let msg = match postman_rx.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    println!("Unable to receive message from Postman channel: {:?}", e);
                    continue
                }
            };

            let stream = match msg.client.stream.try_clone() {
                Ok(stream) => stream,
                Err(e) => {
                    println!("Unable to clone steam: {:?}", e);
                    continue
                }
            };

            let mut writer = BufWriter::new(stream);

            // Write to stream
            match writer.write(msg.content.as_bytes()) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to write to message to stream: {:?}", e);
                    continue
                }
            };

            match writer.flush() {
                Ok(_) => {},
                Err(e) => {
                    // Error handling when client is disconnected
                    println!("Unable to flush stream: {:?}", e);
                    if e.kind() == ErrorKind::BrokenPipe || e.kind() == ErrorKind::ConnectionReset {
                        let unregister_message = RegistrationMessage {
                            client: msg.client.clone(),
                            action: RegistrationAction::LEAVE
                        };

                        // Unregister client
                        match register_tx.send(unregister_message) {
                            Ok(_) => {},
                            Err(e) => {
                                println!("Unable to send unregister message after error: {:?}", e);
                                continue
                            }
                        }
                    }
                    continue
                }
            }
        }
    });
}

pub fn send_message(message: PostmanMessage, postman_tx: Sender<PostmanMessage>) {
    match postman_tx.send(message) {
        Ok(_) => return,
        Err(e) => {
            println!("Unable to write PostmanMessage to the channel: {:?}", e);
            return
        }
    }
}