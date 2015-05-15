use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use message::IncomingMessage;
use message::OutgoingMessage;

pub mod cli;
pub use self::cli::CliAdapter;

pub enum AdapterMsg {
    Outgoing(OutgoingMessage),
    Stop
}

pub trait ChatAdapter {
    fn get_name(&self) -> &str;
    fn process_events(&self) -> Receiver<IncomingMessage>;
}

