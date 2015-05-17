use std::error::Error;
use std::sync::mpsc::SendError;
use std::fmt;

mod echo;
pub use self::echo::EchoHandler;

use message::IncomingMessage;
use message::AdapterMsg;

/// Failure modes for a MessageHandler
#[derive(Debug)]
enum HandlerError {
    /// Failed to send reply
    Reply(SendError<AdapterMsg>),
    /// Other indicates any mode that's not explicitly part of HandlerError
    #[allow(dead_code)]
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

pub type HandlerResult = Result<(), HandlerError>;

/// Implementing a MessageHandler enables responding to IncomingMessages. There
/// are currently very few requirements to creating a handler. The `handle`
/// function receives `IncomingMessage`s. The implementer is responsible for
/// testing whether it's interested in replying. Optionally call `reply` on the
/// IncomingMessage to send a response.
///
/// # Example
///
/// A simple echo handler might look something like the following:
///
/// ```rust
/// struct EchoHandler;
/// impl EchoHandler {
///     pub fn new() -> EchoHandler { EchoHandler }
/// }
///
/// impl MessageHandler for EchoHandler {
///     fn name(&self) -> &str {
///         "echo"
///     }
///
///     fn handle(&self, incoming: &IncomingMessage) -> HandlerError {
///         try!(incoming.reply(incoming.get_contents()));
///     }
/// }
/// ```
///
/// Then attach it to an instance of Chatbot.
///
/// ```rust
/// let bot = Chatbot::new();
///
/// bot.add_adapter(Box::new(EchoHandler::new()));
/// ```
///
pub trait MessageHandler {
    fn name(&self) -> &str;
    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult;
}

