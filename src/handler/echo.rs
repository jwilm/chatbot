use handler::MessageHandler;
use message::IncomingMessage;
use message::OutgoingMessage;

pub struct EchoHandler;

impl EchoHandler {
    pub fn new() -> EchoHandler {
        EchoHandler
    }
}

impl MessageHandler for EchoHandler {
    fn get_name(&self) -> &str {
        "echo"
    }

    fn on_message(&self, message: &IncomingMessage) -> Option<OutgoingMessage> {
        Some(OutgoingMessage::new(message.get_contents().to_owned()))
    }
}
