use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use uuid::Uuid;

use crate::protocol::{part_msg, pong, priv_msg, welcome_reply};
use crate::server::{BroadcastMessage, ChannelMessage};

pub struct Client {
    pub id: Uuid,
    pub stream: TcpStream,
    pub username: String,
    pub channel: Option<String>
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            id: self.id,
            stream: self.stream.try_clone().expect("Unable to clone client's stream"),
            username: self.username.clone(),
            channel: self.channel.clone()
        }
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub fn handle_client(client: TcpStream, broadcast_tx: Sender<BroadcastMessage>, registration_tx: Sender<Client>, channel_tx: Sender<ChannelMessage>) {
    let mut reader = BufReader::new(client.try_clone().unwrap());

    let mut recvd_message = String::new();

    let mut current_client: Option<Client> = Option::None;

    while match reader.read_line(&mut recvd_message) {
        Ok(_) => {
            handle_msg(
                recvd_message.clone(),
                client.try_clone().unwrap(),
                broadcast_tx.clone(),
                registration_tx.clone(),
                channel_tx.clone(),
                &mut current_client
            );

            recvd_message = String::new();
            true
        },
        Err(e) => {
            panic!("{:?}", e);
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

            channel_tx.send(msg);
        }
        "PING" => {
            let mut writer = BufWriter::new(stream.try_clone().unwrap());
            writer.write(pong(String::from(args[1])).as_bytes());
            writer.flush();
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

            broadcast_tx.send(msg);
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

            broadcast_tx.send(msg);

            let msg = ChannelMessage {
                client: current_client_mut.clone().unwrap(),
                channel: String::from(args[1]),
                leave: true
            };

            channel_tx.send(msg);
        }
        _ => {
            println!("Command {} not found", args[0]);
        }
    }

}

fn register_client(stream: TcpStream, username: String, registration_tx: Sender<Client>, current_client: &mut Option<Client>) {
    let client = Client{
        id: Uuid::new_v4(),
        stream: stream.try_clone().unwrap(),
        username: username.clone(),
        channel: None
    };

    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    writer.write(welcome_reply(username).as_bytes());
    writer.flush();

    *current_client = Option::Some(client.clone());

    registration_tx.send(client);
}