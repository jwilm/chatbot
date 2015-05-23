#![deny(unused_must_use)]

//!
//! chatbot is an extensible platform for writing chatbots.
//!
//! Note: the only adapter available at this time is the
//! [`CliAdapter`](adapter/cli/struct.CliAdapter.html). For a complete list of
//! all adapters, see
//! [implementations](adapter/trait.ChatAdapter.html#implementors).
//!
//! ## Handlers and Adapters
//!
//! At its core are the ideas of handlers and adapters. An adapter
//! (implementation of the [`ChatAdapter`](adapter/trait.ChatAdapter.html)
//! trait) enables the push and pull of information from a service like IRC,
//! Slack, web hooks, and etc. Several adapters can be run simultaneously.
//! Cross-service responses are not (currently) supported. For example, messages
//! arriving from IRC can not send a response to Slack.  A handler
//! (implementation of the [`MessageHandler`](handler/trait.MessageHandler.html)
//! trait) can reply to incoming messages. In the current version, the entire
//! message is passed into each handler and it is up to the handler to determine
//! whether it is interested in the message. This is likely to be changed in the
//! near future to simplify implementation of the handlers.
//!
//! ## Chatbot
//!
//! Chatbot is the type which bridges adapters and handlers. Any program which
//! uses chatbot will need to minimally create a Chatbot, add an adapter, add a
//! handler, and call Chatbot [`run`](chatbot/struct.Chatbot.html#method.run).
//! For example, setting up a simple echo server that responds to CLI input:
//!
//! # Examples
//!
//! ```no_run
//! # // bot.run() loops indefinitely
//! use chatbot::Chatbot;
//! use chatbot::adapter::CliAdapter;
//! use chatbot::handler::EchoHandler;
//!
//! let mut bot = Chatbot::new();
//!
//! bot.add_adapter(Box::new(CliAdapter::new()));
//! bot.add_handler(Box::new(EchoHandler::new()));
//!
//! bot.run();
//! ```
//!

extern crate regex;
extern crate hyper;
extern crate rustc_serialize;
extern crate slack;

macro_rules! regex(
    ($s:expr) => (regex::Regex::new($s).unwrap());
);

pub mod chatbot;
pub mod adapter;
pub mod handler;
pub mod message;

pub use chatbot::Chatbot;
