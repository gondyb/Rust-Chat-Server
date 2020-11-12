use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::client_handler::Client;
use crate::protocol::{join_end_members, join_header, join_members, join_message};

pub struct ChannelMessage {
    pub client: Client,
    pub channel: String,
    pub leave: bool
}

#[derive(Clone)]
pub struct Channel {
    pub name: String,
    pub description: String,
    pub clients: Vec<Client>
}

pub struct BroadcastMessage {
    pub content: String,
    pub sender: Client,
    pub channel: String,
    pub send_to_sender: bool
}

pub fn start_server(
    broadcast_tx: Sender<BroadcastMessage>,
    registration_rx: Receiver<Client>,
    broadcast_rx: Receiver<BroadcastMessage>,
    channel_rx: Receiver<ChannelMessage>
) {
    let clients: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));

    let channels: Arc<Mutex<HashMap<String, Channel>>> = Arc::new(Mutex::new(HashMap::new()));

    let rust_channel = Channel {
        name: String::from("#rust"),
        description: String::from("Un endroit pour discuter du rust"),
        clients: Vec::new()
    };

    let java_channel = Channel {
        name: String::from("#java"),
        description: String::from("Un endroit pour discuter du java"),
        clients: Vec::new()
    };

    channels.lock().unwrap().insert(String::from("#rust"), rust_channel);
    channels.lock().unwrap().insert(String::from("#java"), java_channel);

    let _clients = clients.clone();
    let _channels = channels.clone();

    thread::spawn(move || {
        loop {
            let change_channel_message = channel_rx.recv().unwrap();

            if !_channels.lock().unwrap().contains_key(&*change_channel_message.channel) {
                panic!("No channel");
            }

            let mut channels = _channels.lock().unwrap();

            let channel = channels.get_mut(&*change_channel_message.channel).unwrap();

            if change_channel_message.leave == true {
                let mut index_to_remove = 0;
                for i in 0..channel.clients.len(){
                    if *channel.clients.get(i).unwrap() == change_channel_message.client {
                        index_to_remove = i;
                        break;
                    }
                }

                channel.clients.remove(index_to_remove);
                continue
            }

            channel.clients.push(change_channel_message.client.clone());

            let nick = change_channel_message.client.username.clone();
            let domain = change_channel_message.client.stream.local_addr().unwrap().ip().to_string();
            let channel_name = change_channel_message.channel;

            let channel = channels.get(&*channel_name.clone()).unwrap();
            let mut writer = BufWriter::new(change_channel_message.client.stream.try_clone().unwrap());

            let join_msg = join_message(nick.clone(), domain, channel_name.clone());
            writer.write(join_msg.clone().as_bytes());

            let bmsg = BroadcastMessage {
                content: join_msg,
                channel: channel_name.clone(),
                sender: change_channel_message.client.clone(),
                send_to_sender: false
            };

            broadcast_tx.send(bmsg);

            let join_h = join_header(nick.clone(), channel);
            writer.write(join_h.as_bytes());

            let members_msg = join_members(nick.clone(), channel);
            writer.write(members_msg.as_bytes());

            let members_end = join_end_members(nick.clone(), channel);
            writer.write(members_end.as_bytes());

            writer.flush();
        }
    });

    let _clients = clients.clone();

    thread::spawn(move || {
        loop {
            let new_client = registration_rx.recv();
            match new_client {
                Ok(client) => {
                    println!("New client registered: {:?}", client.username.clone());
                    _clients.lock().unwrap().push(client);
                }
                Err(E) => {
                    println!("{:?}", E);
                }
            }
        }
    });

    let _clients = clients.clone();

    let _channels = channels.clone();

    thread::spawn(move || {
        loop {
            let msg = broadcast_rx.recv();
            match msg {
                Ok(m) => {
                    match _channels.lock() {
                        Ok(channels) => {
                            for client in channels.get(&*m.channel.clone()).unwrap().clients.iter() {
                                if *client == m.sender && m.send_to_sender == false {
                                    continue;
                                }
                                let mut writer = BufWriter::new(client.stream.try_clone().unwrap());
                                writer.write(&m.content.as_bytes());
                                writer.flush();
                            }
                        },
                        Err(E) => {
                            println!("{:?}", E);
                        }
                    }
                },
                Err(e) => {
                    println!("Error when receiving message {}", e);
                }
            }
        }
    });
}