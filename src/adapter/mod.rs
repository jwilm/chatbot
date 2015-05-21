use std::sync::mpsc::Receiver;

use message::IncomingMessage;

mod cli;
pub use self::cli::CliAdapter;

mod slack;
pub use self::slack::SlackAdapter;

/// Chatbot is extensible in both message sources and command handling. To add a
/// new message source, create a type that implements the `ChatAdapter` trait.
pub trait ChatAdapter {
    /// The name of the adapter which is used internally as a map key and for
    /// debugging.
    fn get_name(&self) -> &str;

    /// ChatAdapters must implement process_events. What this method does will
    /// vary wildly by adapter. At the very least, it must return a receiver
    /// which the main loop will `recv` on to get messages from the adapter.
    fn process_events(&self) -> Receiver<IncomingMessage>;
}

