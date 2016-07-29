use std::io::{self, Write};
use std::sync::mpsc::Sender;
use std::sync::mpsc::channel;
use std::thread;

use regex::Regex;

use adapter::ChatAdapter;
use message::AdapterMsg;
use message::IncomingMessage;


/// The CliAdapter reads lines from stdin and dispatches them as
/// IncomingMessages to the chatbot.  Replies are printed to stdout. There is
/// currently no extra configuration available to the CliAdapter.
pub struct CliAdapter {
    address_regex: Regex,
}

impl CliAdapter {
    /// create a new CliAdapter
    pub fn new(bot_name: &str) -> CliAdapter {
        CliAdapter {
            address_regex: Regex::new(format!(r"^{}:", bot_name).as_str()).unwrap()
        }
    }
}

impl ChatAdapter for CliAdapter {
    /// name of CliAdapter
    fn get_name(&self) -> &str {
        "cli"
    }

    fn addresser(&self) -> &Regex {
        &self.address_regex
    }

    /// The CliAdapter uses two threads to
    ///
    /// 1.  receive input from stdin and
    /// 2.  listen for messages coming from the main thread. This implementation
    ///     may be horribly inefficient.
    fn process_events(&mut self, tx_incoming: Sender<IncomingMessage>) {
        println!("CliAdapter: process_events");

        let (tx_outgoing, rx_outgoing) = channel();
        let name = self.get_name().to_owned();

        // Read from stdin and send messages to the main loop
        thread::Builder::new().name("Chatbot CLI Reader".to_owned()).spawn(move || {
            loop {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    Ok(len) => {
                        if len == 0 {
                            break;
                        }
                        let msg = IncomingMessage::new(name.to_owned(), None, None, None,
                            line[..(len-1)].to_owned(),
                            tx_outgoing.to_owned());
                        tx_incoming.send(msg).unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                };
            }

            println!("CliAdapter: shutting down");
        }).ok().expect("failed to create stdio reader");

        // process messages from the main loop
        thread::Builder::new().name("Chatbot CLI".to_owned()).spawn(move || {
            loop {
                // TODO don't blindly unwrap
                match rx_outgoing.recv().unwrap() {
                    AdapterMsg::Outgoing(msg) => {
                        io::stdout().write(msg.as_bytes()).unwrap();
                        io::stdout().write(b"\n").unwrap();
                        io::stdout().flush().unwrap();
                    },
                    _ => break
                }
            }
        }).ok().expect("failed to create stdio <-> chatbot proxy");

    }
}

