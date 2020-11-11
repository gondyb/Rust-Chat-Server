use ansi_term::Style;
use ansi_term::Colour::Red;
use chrono::{DateTime, Utc};


pub struct UserMessage {
    pub(crate) username: String,
    pub(crate) content: String,
    pub(crate) date: DateTime<Utc>
}

pub struct SystemMessage {
    pub(crate) content: String,
    pub(crate) date: DateTime<Utc>
}

pub trait Message {
    fn beautify(&self) -> String;
}

impl Message for UserMessage {
    fn beautify(&self) -> String {
        format_args!(
            "[{}] {} {}\r\n",
            self.date.to_rfc2822(),
            Style::new().italic().paint(self.username.clone()),
            self.content
        ).to_string()
    }
}

impl Message for SystemMessage {
    fn beautify(&self) -> String {
        Red.bold().paint(
        format_args!(
                "[{}] {} {}\r\n",
                self.date.to_rfc2822(),
                "SYSTEM",
                self.content
            ).to_string()
        ).to_string()
    }
}
