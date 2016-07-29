//! Contains the `ChatAdapter` trait and several implementations

use std::sync::mpsc::Sender;

use regex::Regex;

use message::IncomingMessage;

mod cli;
pub use self::cli::CliAdapter;

#[cfg(feature = "slack-adapter")]
mod slack;
#[cfg(feature = "slack-adapter")]
pub use self::slack::SlackAdapter;

#[cfg(feature = "irc-adapter")]
mod irc;
#[cfg(feature = "irc-adapter")]
pub use self::irc::IrcAdapter;
#[cfg(feature = "irc-adapter")]
pub use self::irc::IrcConfig;

/// Chatbot is extensible in both message sources and command handling. To add a
/// new message source, create a type that implements the `ChatAdapter` trait.
pub trait ChatAdapter {
    /// The name of the adapter which is used internally as a map key and for
    /// debugging.
    fn get_name(&self) -> &str;

    /// Users are addressed differently in different chat platforms. This allows the adapter to
    /// customize detection forn the bot being addressed.
    fn addresser(&self) -> &Regex;

    /// ChatAdapters must implement process_events. What this method does will
    /// vary wildly by adapter. At the very least, it must generate IncominMessages from its input,
    /// send them via the `Sender` that's passed in. The main loop has the other end of this
    /// receiver. The IncomingMessage must be constructed with a `Sender<OutgoingMessage>` for
    /// which the adapter listens on the Receiver to send messages back to the service.
    fn process_events(&mut self, Sender<IncomingMessage>);
}

