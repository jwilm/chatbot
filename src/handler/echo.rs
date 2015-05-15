use handler::MessageHandler;
use message::IncomingMessage;
use message::OutgoingMessage;
use adapter::AdapterMsg;

pub struct EchoHandler;

impl EchoHandler {
    pub fn new() -> EchoHandler {
        EchoHandler
    }
}

impl MessageHandler for EchoHandler {
    fn name(&self) -> &str {
        "echo"
    }

    fn handle(&self, incoming: &IncomingMessage) {
        let msg = OutgoingMessage::new(incoming.get_contents().to_owned());
        incoming.reply(AdapterMsg::Outgoing(msg));
    }
}
