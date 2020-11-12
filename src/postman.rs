use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::io::{BufWriter, Write};
use crate::client_handler::Client;

pub struct PostmanMessage {
    pub(crate) client: Client,
    pub(crate) content: String
}

pub fn start_postman_thread(postman_rx: Receiver<PostmanMessage>) {
    thread::spawn(move || {
        loop {
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
                    println!("Unable to flush stream: {:?}", e);
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