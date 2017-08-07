//! Contains the MessageHandler trait and handler implementations.

extern crate regex;

use std::error::Error;
use std::fmt;
use std::io;
use std::sync::mpsc::SendError;
use regex::Regex;
use regex::Captures;

use message::IncomingMessage;
use message::AdapterMsg;

/// Failure modes for a MessageHandler
#[derive(Debug)]
pub enum HandlerError {
    /// Failed to send reply
    Reply(SendError<AdapterMsg>),
    /// Other indicates any mode that's not explicitly part of HandlerError
    Other(Box<Error>)
}

impl Error for HandlerError {
    fn description(&self) -> &str {
        match *self {
            HandlerError::Reply(_) => "Failed to send reply because adapter disconnected",
            HandlerError::Other(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            HandlerError::Reply(ref err) => Some(err),
            HandlerError::Other(_) => None
        }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandlerError::Reply(ref err) => write!(f, "Reply error: {}", err),
            HandlerError::Other(ref err) => write!(f, "Other error: {}", err),
        }
    }
}

impl From<SendError<AdapterMsg>> for HandlerError {
    fn from(err: SendError<AdapterMsg>) -> HandlerError {
        HandlerError::Reply(err)
    }
}

impl From<io::Error> for HandlerError {
    fn from(err: io::Error) -> HandlerError {
        HandlerError::Other(Box::new(err))
    }
}

pub type HandlerResult = Result<(), HandlerError>;

/// Implementing a MessageHandler enables responding to IncomingMessages. There
/// are currently very few requirements to creating a handler. The
/// [`handle`](#tymethod.handle) function receives
/// [`IncomingMessage`](../message/struct.IncomingMessage.html)s.  The implementer
/// is responsible for testing whether it's interested in replying. Optionally
/// call [`reply`](../message/struct.IncomingMessage.html#method.reply) on the
/// [`IncomingMessage`](../message/struct.IncomingMessage.html) to send a response.
///
/// # Example
///
/// A simple echo handler might look something like the following:
///
/// ```rust
/// # extern crate chatbot;
/// # extern crate regex;
/// # fn main() {
///
/// use chatbot::handler::MessageHandler;
/// use chatbot::handler::HandlerResult;
/// use chatbot::message::IncomingMessage;
///
/// use regex::Regex;
///
/// struct EchoHandler {
///     regex: Regex
/// }
///
/// impl EchoHandler {
///     fn new() -> EchoHandler {
///         EchoHandler {
///             regex: Regex::new(r".").unwrap()
///         }
///     }
/// }
///
/// impl MessageHandler for EchoHandler {
///     fn name(&self) -> &str {
///         "echo"
///     }
///
///     fn re(&self) -> &Regex {
///         &self.regex
///     }
///
///     fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
///         let response = incoming.get_contents().to_owned();
///         Ok(try!(incoming.reply(response)))
///     }
/// }
/// # }
/// ```
///
pub trait MessageHandler {
    fn name(&self) -> &str;
    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult;
    fn re(&self) -> &Regex;

    /// Uses re() to test whether the handler should process this message.
    fn can_handle(&self, msg: &str) -> bool {
        self.re().is_match(msg)
    }

    /// Uses re() to get capturing groups from a message
    fn get_captures<'a>(&self, msg: &'a str) -> Option<Captures<'a>> {
        self.re().captures(msg)
    }
}


/// A basic response handler
///
/// Provide an re matcher, a name, and a lambda to send simple responses.
pub struct BasicResponseHandler {
    name: String,
    trigger: Regex,
    responder: Box<Fn(Captures, &str) -> Option<String>>
}

impl BasicResponseHandler {
    pub fn new<F>(name: &str, trigger: &str, responder: F) -> BasicResponseHandler
        where F: Fn(Captures, &str) -> Option<String>  + 'static {

        BasicResponseHandler {
            name: name.to_owned(),
            responder: Box::new(responder),
            trigger: regex!(trigger)
        }
    }
}

impl MessageHandler for BasicResponseHandler {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn re(&self) -> &Regex {
        &self.trigger
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        let ref make_response = self.responder;
        let msg = incoming.get_contents();

        match make_response(self.get_captures(msg).unwrap(), msg) {
            Some(response) => try!(incoming.reply(response)),
            None => ()
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use handler::BasicResponseHandler;
    use handler::MessageHandler;
    use message::IncomingMessage;
    use message::AdapterMsg;

    #[test]
    fn test_basic_response_echo() {
        let handler = BasicResponseHandler::new("EchoHandler", r"echo", |_, msg| {
            Some(msg.to_owned())
        });

        let test_msg = "echo this message";
        assert!(handler.can_handle(test_msg));
        let (tx, rx) = channel();
        let msg = IncomingMessage::new(handler.name().to_owned(),
            None, None, None, test_msg.to_owned(), tx);
        handler.handle(&msg).unwrap();
        match rx.recv().unwrap() {
            AdapterMsg::Outgoing(out) => assert_eq!(out.as_ref(), test_msg),
            _ => unreachable!()
        }
    }

    #[test]
    fn test_basic_responder_ping() {
        let handler = BasicResponseHandler::new("PingHandler", r"ping", |_, _| {
            Some("pong".to_owned())
        });

        assert!(handler.can_handle("ping"));
        let (tx, rx) = channel();
        let msg = IncomingMessage::new(handler.name().to_owned(),
            None, None, None, "ping".to_owned(), tx);
        handler.handle(&msg).unwrap();
        match rx.recv().unwrap() {
            AdapterMsg::Outgoing(out) => assert_eq!(out.as_ref(), "pong"),
            _ => unreachable!()
        }
    }
}
