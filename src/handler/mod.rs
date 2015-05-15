pub mod echo;
pub use self::echo::EchoHandler;

use message::IncomingMessage;
use message::OutgoingMessage;

pub trait MessageHandler {
    fn get_name(&self) -> &str;
    fn on_message(&self, payload: &IncomingMessage) -> Option<OutgoingMessage>;
}

