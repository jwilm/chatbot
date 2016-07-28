use regex::{Regex, RegexBuilder};
use std::sync::mpsc::channel;

use adapter::ChatAdapter;
use handler::MessageHandler;
use message::IncomingMessage;

fn dispatch(handlers: &Vec<Box<MessageHandler>>, msg: &IncomingMessage) {
    let msg_str = msg.get_contents();

    for handler in handlers {
        if handler.can_handle(msg_str) {
            match handler.handle(msg) {
                Err(e) => {
                    println!("Error in handler `{}`", handler.name());
                    println!("{:?}", e);
                    println!("The incoming message was {}", msg_str);

                    // TODO remove handler?
                },
                _ => ()
            }
        }
    }
}

/// The Chatbot is the central data structure of the chatbot platform. It contains a `run` method
/// which listens for messages from adapters and routes them to handlers. Any program which uses
/// chatbot will need to minimally create a Chatbot, add an adapter, add a handler, and call Chatbot
/// [`run`](chatbot/struct.Chatbot.html#method.run).
pub struct Chatbot {
    name: String,
    adapters: Vec<Box<ChatAdapter>>,
    handlers: Vec<Box<MessageHandler>>,
    addressed_handlers: Vec<Box<MessageHandler>>,
    addresser: Regex
}

impl Chatbot {
    /// Create a new chatbot instance.
    ///
    /// The name provided here will be used for message filtering and in handlers for whatever they
    /// want. You'll want to make your binding mutable so you can call `add_adapter` and
    /// `add_handler`.
    pub fn new(name: &str) -> Chatbot {
        let addresser_str = format!(r"^\s*@?{}:?", name);
        let addresser = RegexBuilder::new(&addresser_str[..])
                                     .case_insensitive(true)
                                     .compile();

        Chatbot {
            name: name.to_owned(),
            adapters: Vec::new(),
            handlers: Vec::new(),
            addressed_handlers: Vec::new(),
            addresser: addresser.unwrap()
        }
    }

    /// Return the name provided in initialization
    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    /// Return the regular expression of what the bot will be addressed by
    pub fn get_addresser(&self) -> &Regex {
        &self.addresser
    }

    /// Add a ChatAdapter to the bot
    ///
    /// Add as many adapters as you like. The IncomingMessages sent by adapters are made available
    /// to all handlers regardless of how many adapters exist. The IncomingMessage.reply method
    /// makes sure the response is sent back to the adapter from whence the message came.
    pub fn add_adapter<T>(&mut self, adapter: T)
        where T: ChatAdapter + 'static
    {
        println!("Adding adapter {}", adapter.get_name());
        self.adapters.push(Box::new(adapter))
    }

    /// Add a MessageHandler to the bot
    ///
    /// The more handlers you have the more useful your bot becomes (for potentially loose
    /// definitions of useful :P). Check out the handler! macro for making simple handlers and also
    /// see the `MessageHandler` implementors section for a list of built in handlers.
    pub fn add_handler<T>(&mut self, handler: T)
        where T: MessageHandler + 'static
    {
        println!("Adding handler {}", handler.name());
        self.handlers.push(Box::new(handler))
    }

    /// Add a MessageHandler, that requires the bot to be addressed, to the bot
    pub fn add_addressed_handler<T>(&mut self, handler: T)
        where T: MessageHandler + 'static
    {
        println!("Adding handler {}", handler.name());
        self.addressed_handlers.push(Box::new(handler))
    }

    /// Start processing messages
    ///
    /// Call process_events on all of the adapters and `recv` on the `IncomingMessage` channel.
    /// Distribute IncomingMessages to list of handlers.
    pub fn run(&self) {
        let adapters_len = self.adapters.len();
        let handlers_len = self.handlers.len() + self.addressed_handlers.len();

        assert!(adapters_len > 0);
        assert!(handlers_len > 0);

        println!("Chatbot: {} adapters", adapters_len);
        println!("Chatbot: {} handlers", handlers_len);

        let (incoming_tx, incoming_rx) = channel();

        for adapter in &self.adapters {
            adapter.process_events(self, incoming_tx.clone());
        }

        loop {
            // Get message from adapter
            let msg = match incoming_rx.recv() {
                Ok(msg) => msg,
                Err(_) => break
            };

            // Only dispatch to addressed handlers when bot is addressed
            if self.addresser.is_match(msg.get_contents()) {
                dispatch(&self.addressed_handlers, &msg);
            }

            // Always dispatch to global handlers
            dispatch(&self.handlers, &msg);
        }

        println!("chatbot shutting down");
    }
}

#[cfg(test)]
mod tests {
    use chatbot::Chatbot;
    use adapter::CliAdapter;

    #[test]
    fn test_create_chatbot() {
        let name = "testbot";
        let bot = Chatbot::new("testbot");
        assert_eq!(bot.get_name(), name);
    }

    #[test]
    fn test_chatbot_add_adapter() {
        let mut bot = Chatbot::new("testbot");
        let cli = CliAdapter::new();
        bot.add_adapter(cli);
    }

    #[test]
    fn test_chatbot_add_handler() {
        let mut bot = Chatbot::new("testbot");
        let echo = handler!("EchoHandler", r"echo .+", |_, msg| {
            Some(msg.to_owned())
        });
        bot.add_handler(echo);
    }

    #[test]
    fn test_chatbot_address_matches() {
        let bot = Chatbot::new("testbot");
        let addresser = bot.get_addresser();

        assert!(!addresser.is_match("testbotping"),
                "Shouldn't match bot name without a break (space, colon, comma)");
        assert!(!addresser.is_match("@testbotping"),
                "Shouldn't match bot name with '@' sign minus a break (space, colon, comma)");

        // Space separation
        assert!(addresser.is_match("testbot ping"), "Should match bot name with space");
        assert!(addresser.is_match("@testbot ping"),
                "Should match bot name with '@' sign plus '@' space");

        // Colon separation
        assert!(addresser.is_match("testbot:ping"), "Should match bot name with colon");
        assert!(addresser.is_match("testbot: ping"), "Should match bot name with colon and space");
        assert!(addresser.is_match("@testbot:ping"),
                "Should match bot name with '@' sign plus colon");
        assert!(addresser.is_match("@testbot: ping"),
                "Should match bot name with '@' sign plus colon and space");

        // Comma separation
        assert!(addresser.is_match("testbot,ping"), "Should match bot name with comma");
        assert!(addresser.is_match("testbot, ping"), "Should match bot name with comma and space");
        assert!(addresser.is_match("@testbot,ping"),
                "Should match bot name with '@' sign plus comma");
        assert!(addresser.is_match("@testbot, ping"),
                "Should match bot name with '@' sign plus comma and space");
    }
}
