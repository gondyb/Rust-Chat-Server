use crate::server::Channel;

fn generate_response(code: u16, nick: String, content: String) -> String {
    format!(":guyot-gondange.fr {:03} {} :{}\r\n", code, nick, content)
}

pub fn welcome_reply(nick: String) -> String {
    generate_response(001, nick, String::from("Bienvenue sur notre serveur sûr et efficace !"))
}

pub fn join_message(nick: String, domain: String, channel: String) -> String {
    format!(":{}!{}@{} JOIN {}\r\n", nick, nick, domain, channel)
}

pub fn join_header(nick: String, channel: &Channel) -> String {
    format!(":guyot-gondange.fr {:03} {} {} :{}\r\n", 332, nick, channel.name, channel.description)
}

pub fn join_members(nick: String, channel: &Channel) -> String {

    let mut connected_clients_string = String::new();

    for  client in channel.clients.clone() {
        connected_clients_string.push_str(&*client.username);
        connected_clients_string.push_str(" ");
    }

    format!(":guyot-gondange.fr {:03} {} = {} :{}\r\n", 353, nick, channel.name, connected_clients_string)
}

pub fn join_end_members(nick: String, channel: &Channel) -> String {
    format!(":guyot-gondange.fr {:03} {} {} :{}\r\n", 366, nick, channel.name, "End of NAMES list")

}

pub fn pong(domain: String) -> String {
    format!("PONG guyot-gondange.fr {}", domain)
}

pub fn priv_msg(nick: String, domain: String, channel: String, content: String) -> String {
    format!(":{}!{}@{} PRIVMSG {} :{}\r\n", nick, nick, domain, channel, content)
}

pub fn part_msg(nick: String, domain: String, channel: String, content: String) -> String {
    format!(":{}!{}@{} PART {} :{}\r\n", nick, nick, domain, channel, content)
}