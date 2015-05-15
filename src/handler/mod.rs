pub mod echo;
pub use self::echo::EchoHandler;

use message::IncomingMessage;
use message::OutgoingMessage;

pub trait MessageHandler {
    fn name(&self) -> &str;
    fn handle(&self, incoming: &IncomingMessage);
}

