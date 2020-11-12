use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use uuid::Uuid;

use crate::protocol::{part_msg, pong, priv_msg};
use crate::channels::{ChannelMessage, send_channel_message};
use crate::broadcast::{BroadcastMessage, send_broadcast_message};

pub struct Client {
    pub id: Uuid,
    pub stream: TcpStream,
    pub username: String,
    pub domain: String,
    pub channel: Option<String>
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            id: self.id,
            stream: self.stream.try_clone().expect("Unable to clone client's stream"),
            username: self.username.clone(),
            domain: self.domain.clone(),
            channel: self.channel.clone()
        }
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub fn handle_client(
    client: TcpStream,
    broadcast_tx: Sender<BroadcastMessage>,
    registration_tx: Sender<Client>,
    channel_tx: Sender<ChannelMessage>
) {
    let stream = match client.try_clone() {
        Ok(stream) => stream,
        Err(e) => {
            println!("Unable to clone stream: {:?}", e);
            return
        }
    };

    let mut reader = BufReader::new(stream);

    let mut received_message = String::new();

    let mut current_client: Option<Client> = Option::None;

    while match reader.read_line(&mut received_message) {
        Ok(_) => {
            handle_msg(
                received_message.clone(),
                client.try_clone().unwrap(),
                broadcast_tx.clone(),
                registration_tx.clone(),
                channel_tx.clone(),
                &mut current_client
            );

            received_message = String::new();
            true
        },
        Err(e) => {
            println!("Unable to read message fro client: {:?}", e);
            // TODO: Client disconnected
            false
        }
    } {}
}

fn handle_msg(
    msg: String,
    stream: TcpStream,
    broadcast_tx: Sender<BroadcastMessage>,
    registration_tx: Sender<Client>,
    channel_tx: Sender<ChannelMessage>,
    current_client_mut: &mut Option<Client>
    ) {
    println!("{:?}", msg);

    let args: Vec<&str> = msg.split_whitespace().collect();
    let body: Vec<&str> = msg.split(":").collect();

    match args[0] {
       "NICK" => {
            register_client(
                stream,
                String::from(args[1]),
                registration_tx.clone(),
                current_client_mut
            );
        }
        "JOIN" => {
            let msg = ChannelMessage {
                client: current_client_mut.clone().unwrap(),
                channel: String::from(args[1]),
                leave: false
            };

            match channel_tx.send(msg) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to send message to channel channel: {:?}", e);
                    return
                }
            };
        }
        "PING" => {
            // Message is sent without postman, because the message can be received even if client
            // has not registered yet.
            let mut writer = BufWriter::new(stream.try_clone().unwrap());
            match writer.write(pong(String::from(args[1])).as_bytes()) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to send back pong message: {:?}", e);
                    return
                }
            };

            match writer.flush() {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to flush pong message: {:?}", e);
                    return
                }
            }
        }
        "PRIVMSG" => {
            let sender = current_client_mut.clone().unwrap();
            let content = priv_msg(
                sender.username.clone(),
                sender.stream.local_addr().unwrap().ip().to_string(),
                String::from(args[1]),
                String::from(body[1]).replace('\r', "").replace('\n', "")
            );

            let msg = BroadcastMessage {
                content,
                channel: String::from(args[1]),
                sender,
                send_to_sender: false
            };

            send_broadcast_message(msg, broadcast_tx.clone());
        }
        "PART" => {
            let sender = current_client_mut.clone().unwrap();
            let content = part_msg(
                sender.username.clone(),
                sender.stream.local_addr().unwrap().ip().to_string(),
                String::from(args[1]),
                String::from(body[1]).replace('\r', "").replace('\n', "")
            );

            let msg = BroadcastMessage {
                content,
                channel: String::from(args[1]),
                sender,
                send_to_sender: true
            };

            send_broadcast_message(msg, broadcast_tx.clone());

            let msg = ChannelMessage {
                client: current_client_mut.clone().unwrap(),
                channel: String::from(args[1]),
                leave: true
            };

            send_channel_message(msg, channel_tx.clone());
        }
        _ => {
            println!("Command {} not found", args[0]);
        }
    }

}

fn register_client(stream: TcpStream, username: String, registration_tx: Sender<Client>, current_client: &mut Option<Client>) {
    let local_addr = match stream.local_addr() {
        Ok(local_addr) => local_addr,
        Err(e) => {
            println!("Unable to retrieve local_addr from stream: {:?}", e);
            return
        }
    };


    let client = Client{
        id: Uuid::new_v4(),
        stream: stream.try_clone().unwrap(),
        username: username.clone(),
        domain: local_addr.ip().to_string(),
        channel: None
    };

    *current_client = Option::Some(client.clone());

    match registration_tx.send(client) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send registration message to channel: {:?}", e);
            return;
        }
    };
}