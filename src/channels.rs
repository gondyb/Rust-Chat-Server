use std::sync::mpsc::{Receiver, Sender};
use crate::client_handler::Client;
use std::thread;
use std::io::{BufWriter, Write};
use crate::protocol::{join_message, join_header, join_members, join_end_members};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::broadcast::BroadcastMessage;

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

pub fn start_channels_thread(
    channels: Arc<Mutex<HashMap<String, Channel>>>,
    channel_rx: Receiver<ChannelMessage>, broadcast_tx: Sender<BroadcastMessage>
) {

    init_default_channels(channels.clone());

    thread::spawn(move || {
        loop {
            let change_channel_message = match channel_rx.recv() {
                Ok(message) => message,
                Err(e) => {
                    println!("Unable to read from channel channel: {:?}", e);
                    continue
                }
            };

            let mut channels = match channels.lock() {
                Ok(channels) => channels,
                Err(e) => {
                    println!("Unable to acquire channels lock: {:?}", e);
                    continue
                }
            };

            let channel = match channels.get_mut(&*change_channel_message.channel) {
                Some(channel) => channel,
                _ => {
                    println!("Channel {} doesn't exist!", &*change_channel_message.channel);
                    // TODO: Send back IRC error message
                    continue
                }
            };

            if change_channel_message.leave == true {
                // User wants to leave the channel
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

            // User wants to join the channel

            channel.clients.push(change_channel_message.client.clone());

            let client = change_channel_message.client.clone();
            let channel_name = change_channel_message.channel;

            let join_msg = join_message(
                client.username.clone(),
                client.domain.clone(),
                channel_name.clone()
            );

            send_synchronous_message(client.clone(), join_msg.clone());

            let broadcast_message = BroadcastMessage {
                content: join_msg,
                channel: channel_name.clone(),
                sender: change_channel_message.client.clone(),
                send_to_sender: false
            };

            match broadcast_tx.send(broadcast_message) {
                Ok(_) => {},
                Err(e) => {
                    println!("Unable to send broadcast channel message: {:?}", e);
                    return;
                }
            };

            let join_header = join_header(client.username.clone(), channel);
            send_synchronous_message(client.clone(), join_header);

            let members_msg = join_members(client.username.clone(), channel);
            send_synchronous_message(client.clone(), members_msg);

            let members_end = join_end_members(client.username.clone(), channel);
            send_synchronous_message(client.clone(), members_end);
        }
    });
}

fn init_default_channels(channels: Arc<Mutex<HashMap<String, Channel>>>) {
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
}

fn send_synchronous_message(client: Client, message: String) {
    let mut writer = BufWriter::new(client.stream);
    match writer.write(message.as_bytes()) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send channel message: {:?}", e);
            return
        }
    };

    match writer.flush() {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to flush channel message {:?}", e);
            return
        }
    };
}

pub fn send_channel_message(message: ChannelMessage, channel_tx: Sender<ChannelMessage>) {
    match channel_tx.send(message) {
        Ok(_) => {},
        Err(e) => {
            println!("Unable to send channel channel message: {:?}",e);
            return
        }
    }
}