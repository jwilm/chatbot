extern crate regex;

use regex::Regex;

use handler::MessageHandler;
use handler::HandlerResult;
use message::IncomingMessage;

/// A standard echo handler. It handles every incoming message by replying with
/// a copy of its contents.
pub struct EchoHandler {
    regex: Regex
}

impl EchoHandler {
    pub fn new() -> EchoHandler {
        EchoHandler {
            regex: regex!(r".")
        }
    }
}

impl MessageHandler for EchoHandler {
    fn name(&self) -> &str {
        "echo"
    }

    fn re(&self) -> &Regex {
        &self.regex
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        Ok(try!(incoming.reply(incoming.get_contents().to_owned())))
    }
}
