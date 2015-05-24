extern crate regex;

use regex::Regex;

use handler::MessageHandler;
use handler::HandlerResult;
use message::IncomingMessage;

/// Respond to "ping" messages with "pong"
pub struct PingHandler {
    regex: Regex
}

impl PingHandler {
    pub fn new() -> PingHandler {
        PingHandler {
            regex: regex!(r"ping")
        }
    }
}

impl MessageHandler for PingHandler {
    fn name(&self) -> &str {
        "ping"
    }

    fn re(&self) -> &Regex {
        &self.regex
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        if self.can_handle(incoming.get_contents()) {
            try!(incoming.reply("pong".to_owned()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use handler::PingHandler;
    use message::IncomingMessage;
    use message::AdapterMsg;
    use handler::MessageHandler;
    use std::sync::mpsc::channel;

    #[test]
    fn test_can_handle() {
        let handler = PingHandler::new();
        assert!(handler.can_handle("ping"));
    }

    #[test]
    fn test_response_contents() {
        let handler = PingHandler::new();
        let (tx, rx) = channel();
        let msg = IncomingMessage::new(handler.name().to_owned(),
            None, None, None, "ping".to_owned(), tx);
        handler.handle(&msg).unwrap();
        let adapter_msg = rx.recv().unwrap();
        match adapter_msg {
            AdapterMsg::Outgoing(out) => {
                assert_eq!(out.as_ref(), "pong");
            },
            _ => unreachable!()
        }
    }
}
