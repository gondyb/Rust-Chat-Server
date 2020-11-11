use std::sync::mpsc::{Sender};
use std::net::{TcpStream, Shutdown};
use std::io::{Write, BufReader, BufWriter, BufRead};
use ansi_term::Colour;
use ansi_term::Colour::RGB;
use rand::Rng;
use crate::messages::{UserMessage, SystemMessage};
use crate::messages::Message;
use chrono::{Utc};


fn generate_random_color() -> Colour {
    //Gives a random color for each user
    let mut rng = rand::thread_rng();
    let rng_r = rng.gen_range(0, 255);
    let rng_g = rng.gen_range(0, 255);
    let rng_b = rng.gen_range(0, 255);

    RGB(rng_r, rng_g, rng_b)
}

pub fn handle_client(stream: TcpStream, sender : Sender<String>) {
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    writer.write(b"Welcome to the chat room!\n").unwrap();
    writer.write(b"Please enter your name: \n").unwrap();
    writer.flush().unwrap();

    let mut name = String::new();
    reader.read_line(&mut name).unwrap();
    name.trim_end();
    name.truncate(name.len() - 2);

    let dt = Utc::now();
    let welcome_message = SystemMessage{
        content: format_args!("Welcome {}!", name.clone()).to_string(),
        date: dt
    };

    sender.send(welcome_message.beautify());

    let user_color = generate_random_color();

    let mut string_message = String::new();
    'read_loop: while match reader.read_line(&mut string_message) {
        Ok(_) => {
            if string_message.is_empty() {
                let dt = Utc::now();
                let welcome_message = SystemMessage{
                    content: format_args!("Goodbye {}!", name.clone()).to_string(),
                    date: dt
                };

                sender.send(welcome_message.beautify());
                stream.shutdown(Shutdown::Both).unwrap();
                break 'read_loop;
            }
            let dt = Utc::now();
            let msg = UserMessage{
                content: string_message,
                username: name.clone(),
                date: dt
            };

            sender.send(user_color.paint(msg.beautify()).to_string()).unwrap();
            string_message = String::new();
            true
        },
        Err(_) => {
            let dt = Utc::now();
            let welcome_message = SystemMessage{
                content: format_args!("Goodbye {}!", name.clone()).to_string(),
                date: dt
            };

            sender.send(welcome_message.beautify());
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}