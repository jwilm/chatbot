use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use handler::IncomingMessage;
use handler::OutgoingMessage;

pub enum AdapterMsg {
    Outgoing(OutgoingMessage),
    Stop
}

pub trait ChatAdapter {
    fn get_name(&self) -> &str;
    fn process_events(&self) -> (Sender<AdapterMsg>, Receiver<IncomingMessage>);
}
