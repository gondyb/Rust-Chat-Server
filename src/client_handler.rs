use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use uuid::Uuid;

use crate::protocol::{pong, priv_msg};
use crate::channels::{ChannelMessage, send_channel_message};
use crate::broadcast::{BroadcastMessage, send_broadcast_message};
use std::thread;
use crate::registration::{RegistrationMessage, RegistrationAction};

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

// Handles client messages and dispatches accordingly
pub fn start_client_thread(
    client: TcpStream,
    broadcast_tx: Sender<BroadcastMessage>,
    registration_tx: Sender<RegistrationMessage>,
    channel_tx: Sender<ChannelMessage>
) {
    thread::spawn(move || {
        let stream = match client.try_clone() {
            Ok(stream) => stream,
            Err(e) => {
                println!("Unable to clone stream: {:?}", e);
                return
            }
        };

        let mut reader = BufReader::new(stream);

        let mut received_message = String::new();

        // Client can be either registered or not
        let mut current_client: Option<Client> = Option::None;

        loop {
            // Read messages
            match reader.read_line(&mut received_message) {
                Ok(_) => {
                    let stream = match client.try_clone() {
                        Ok(stream) => stream,
                        Err(e) => {
                            println!("Error when cloning stream to handle messages: {:?}", e);
                            continue
                        }
                    };

                    let connected = dispatch_message(
                        received_message.clone(),
                        stream,
                        broadcast_tx.clone(),
                        registration_tx.clone(),
                        channel_tx.clone(),
                        &mut current_client
                    );

                    if connected == false {
                        break;
                    }

                    received_message = String::new();
                },
                Err(e) => {
                    println!("Unable to read message fro client: {:?}", e);
                    // TODO: Client disconnected
                    continue
                }
            }
        }

        drop(client);
    });
}

fn dispatch_message(
    msg: String,
    stream: TcpStream,
    broadcast_tx: Sender<BroadcastMessage>,
    registration_tx: Sender<RegistrationMessage>,
    channel_tx: Sender<ChannelMessage>,
    current_client_mut: &mut Option<Client>
) -> bool {
    println!("Received message: {:?}", msg);

    // IRC Message format (RFC 1459): "COMMAND ARG1 ARG2...ARGN :BODY"
    let args: Vec<&str> = msg.split_whitespace().collect();
    let body: Vec<&str> = msg.split(":").collect();

    match args[0] {
        // Client wants to register
       "NICK" => {
            register_client(
                stream,
                String::from(args[1]),
                registration_tx.clone(),
                current_client_mut
            );
        }
        // Client wants to join a channel
        "JOIN" => {
            let current_client = match current_client_mut {
                Some(client) => client,
                _ => {
                    println!("Client not registered! Ignoring message...");
                    return true
                }
            };

            let msg = ChannelMessage {
                client: current_client.clone(),
                channel: Some(String::from(args[1])),
                body: Option::None,
                leave: false,
                all_channels: false
            };

            match channel_tx.send(msg) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to send message to channel channel: {:?}", e);
                    return true
                }
            };
        }
        // Clients wants to check if connectio still alive
        "PING" => {
            // Message is sent without postman, because the message can be received even if client
            // has not registered yet.
            let stream = match stream.try_clone() {
                Ok(stream) => stream,
                Err(e) => {
                    println!("Unable to clone TcpStream: {:?}", e);
                    return true
                }
            };

            let mut writer = BufWriter::new(stream);
            match writer.write(pong(String::from(args[1])).as_bytes()) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to send back pong message: {:?}", e);
                    return true
                }
            };

            match writer.flush() {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to flush pong message: {:?}", e);
                    return true
                }
            }
        }
        // Client sends a message
        // TODO: Add support for direct PRIVMSG
        "PRIVMSG" => {
            let current_client = match current_client_mut {
                Some(client) => client,
                _ => {
                    println!("Client not registered! Ignoring message...");
                    return true
                }
            };

            let sender = current_client.clone();
            let content = priv_msg(
                sender.username.clone(),
                sender.domain.clone(),
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
        // Client wants to leave a channel
        "PART" => {
            let current_client = match current_client_mut {
                Some(client) => client,
                _ => {
                    println!("Client not registered! Ignoring message...");
                    return true
                }
            };

            let body = Option::Some(
                String::from(body[1]).
                    replace('\n', "").
                    replace('\r', "")
            );

            let msg = ChannelMessage {
                client: current_client.clone(),
                channel: Some(String::from(args[1])),
                body,
                leave: true,
                all_channels: false
            };

            send_channel_message(msg, channel_tx.clone());
        }
        "QUIT" => {
            let current_client = match current_client_mut {
                Some(client) => client,
                _ => {
                    println!("Client not registered! Ignoring message...");
                    return true
                }
            };
            unregister_client(current_client.clone(), registration_tx.clone());

            return false
        }
        _ => {
            println!("Command {} not found", args[0]);
        }
    }

    return true

}

fn register_client(stream: TcpStream, username: String, registration_tx: Sender<RegistrationMessage>, current_client: &mut Option<Client>) {
    let local_addr = match stream.local_addr() {
        Ok(local_addr) => local_addr,
        Err(e) => {
            println!("Unable to retrieve local_addr from stream: {:?}", e);
            return
        }
    };

    let stream = match stream.try_clone() {
        Ok(stream) => stream,
        Err(e) => {
            println!("Unable to clone stream: {:?}", e);
            return
        }
    };

    let client = Client{
        id: Uuid::new_v4(),
        stream,
        username: username.clone(),
        domain: local_addr.ip().to_string(),
        channel: None
    };

    *current_client = Option::Some(client.clone());

    let registration_message = RegistrationMessage {
        client,
        action: RegistrationAction::REGISTER
    };

    match registration_tx.send(registration_message) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send registration message to channel: {:?}", e);
            return;
        }
    };
}

fn unregister_client(client: Client, registration_tx: Sender<RegistrationMessage>){
    let unregister_message = RegistrationMessage {
        client: client.clone(),
        action: RegistrationAction::LEAVE
    };

    match registration_tx.send(unregister_message) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send unregistration message to channel: {:?}", e);
            return
        }
    }
}