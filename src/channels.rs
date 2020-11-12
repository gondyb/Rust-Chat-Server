use std::sync::mpsc::{Receiver, Sender};
use crate::client_handler::Client;
use std::thread;
use std::io::{BufWriter, Write};
use crate::protocol::{join_message, join_header, join_members, join_end_members, part_msg};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::broadcast::{BroadcastMessage, send_broadcast_message};
use crate::postman::{PostmanMessage, send_message};

pub struct ChannelMessage {
    pub client: Client,
    pub channel: Option<String>,
    pub body: Option<String>,
    pub leave: bool,
    pub all_channels: bool
}

#[derive(Clone)]
pub struct Channel {
    pub name: String,
    pub description: String,
    pub clients: Vec<Client>
}

pub fn start_channels_thread(
    channels: Arc<Mutex<HashMap<String, Channel>>>,
    channel_rx: Receiver<ChannelMessage>,
    broadcast_tx: Sender<BroadcastMessage>,
    postman_tx: Sender<PostmanMessage>
) {

    // Create two channels: #rust and #java
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

            // User wants to leave channel
            if change_channel_message.leave == true {
                leave_channel(
                    change_channel_message,
                    broadcast_tx.clone(),
                    postman_tx.clone(),
                    channels.clone()
                );
                continue
            }

            let mut channels = match channels.lock() {
                Ok(channels) => channels,
                Err(e) => {
                    println!("Unable to acquire channels lock: {:?}", e);
                    continue
                }
            };

            // User wants to join the channel

            let channel_name = match change_channel_message.channel {
                Some(channel_name) => channel_name,
                _ => {
                    println!("Cannot join channel without name!");
                    return
                }
            };

            let channel = match channels.get_mut(&*channel_name) {
                Some(channel) => channel,
                _ => {
                    println!("Channel {} doesn't exist!", channel_name);
                    // TODO: Send back IRC error message
                    continue
                }
            };

            // Add client to connected clients
            channel.clients.push(change_channel_message.client.clone());

            let client = change_channel_message.client.clone();
            let channel_name = channel_name;

            let join_msg = join_message(
                client.username.clone(),
                client.domain.clone(),
                channel_name.clone()
            );
            // We need to send a sync message, otherwise, the client may receive the channel members
            // before knowing that they successfully joined the channel
            send_synchronous_message(client.clone(), join_msg.clone());

            // Say to other users that someone joined the channel
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

            // Sens the user list and channel description to the client
            let join_header = join_header(client.username.clone(), channel);
            send_synchronous_message(client.clone(), join_header);

            let members_msg = join_members(client.username.clone(), channel);
            send_synchronous_message(client.clone(), members_msg);

            let members_end = join_end_members(client.username.clone(), channel);
            send_synchronous_message(client.clone(), members_end);
        }
    });
}

fn leave_channel(
    change_channel_message: ChannelMessage,
    broadcast_tx: Sender<BroadcastMessage>,
    postman_tx: Sender<PostmanMessage>,
    channels: Arc<Mutex<HashMap<String, Channel>>>,
) {
    // User wants to leave the channel
    let sender = change_channel_message.client.clone();

    // Called when a client unregisters or has a connection error
    if change_channel_message.all_channels == true {
        unregister_from_all_channels(sender, channels, broadcast_tx);
        return
    }

    let channel_to_leave = match change_channel_message.channel.clone() {
        Some(channel) => channel,
        _ => {
            panic!("There should be a channel name");
        }
    };

    let mut channels = match channels.lock() {
        Ok(channels) => channels,
        Err(e) => {
            println!("Unable to acquire channels lock: {:?}", e);
            return
        }
    };

    let channel = match channels.get_mut(&*channel_to_leave) {
        Some(channel) => channel,
        _ => {
            println!("Unable to find given channel name: {:?}", &*channel_to_leave);
            return
        }
    };

    let body = match change_channel_message.body.clone() {
        Some(body) => body,
        _ => String::from("Bye bye")
    };

    let content = part_msg(
        sender.username.clone(),
        sender.domain.clone(),
        String::from(channel_to_leave.clone()),
        String::from(body).replace('\r', "").replace('\n', "")
    );

    // Send message to everyone that a user left
    let msg = BroadcastMessage {
        content: content.clone(),
        channel: channel_to_leave.clone(),
        sender: sender.clone(),
        send_to_sender: false
    };

    send_broadcast_message(msg, broadcast_tx.clone());

    // Send to the client that it left
    let postman_message = PostmanMessage {
        client: sender.clone(),
        content: content.clone()
    };

    send_message(postman_message, postman_tx);

    // Remove client from connected clients
    channel.clients.retain(|c| c.clone() != change_channel_message.client.clone());
}

// Function called to unregister client from every channel (ie. when the connection breaks)
fn unregister_from_all_channels(sender: Client, channels: Arc<Mutex<HashMap<String, Channel>>>, broadcast_tx: Sender<BroadcastMessage>) {
    let mut channels = match channels.lock() {
        Ok(channels) => channels,
        Err(e) => {
            println!("Unable to acquire channel lock: {:?}", e);
            return
        }
    };

    for channel in channels.values_mut() {
        // Check if client in channel
        if !channel.clone().clients.contains(&sender.clone()) {
            continue
        }

        let content = part_msg(
            sender.username.clone(),
            sender.domain.clone(),
            String::from(channel.clone().name),
            String::from("User left")
        );

        let msg = BroadcastMessage {
            content: content.clone(),
            channel: channel.clone().name,
            sender: sender.clone(),
            send_to_sender: false
        };

        // Say to every other client that client disconnected
        send_broadcast_message(msg, broadcast_tx.clone());

        // Remove client for channel's client vector
        channel.clients.retain(|c| c != &sender.clone());
    }
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

    let mut channels = match channels.lock() {
        Ok(channels) => channels,
        Err(e) => {
            println!("Unable to acquire channels lock: {:?}", e);
            return
        }
    };

    channels.insert(String::from("#rust"), rust_channel);
    channels.insert(String::from("#java"), java_channel);
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