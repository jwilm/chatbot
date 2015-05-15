use std::sync::mpsc::Select;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread;
use std::io;
use std::io::Write;

use message::IncomingMessage;
use message::AdapterMsg;

use adapter::ChatAdapter;

/// The CliAdapter reads lines from stdin and dispatches them as
/// IncomingMessages to the chatbot.  Replies are printed to stdout. There is
/// currently no extra configuration available to the CliAdapter.
pub struct CliAdapter;

impl CliAdapter {
    /// create a new CliAdapter
    pub fn new() -> CliAdapter {
        CliAdapter
    }
}

impl ChatAdapter for CliAdapter {
    /// name of CliAdapter
    fn get_name(&self) -> &str {
        "cli"
    }

    /// The CliAdapter uses two threads to
    ///
    /// 1.  receive input from stdin and
    /// 2.  listen for messages coming from the main thread. This implementation
    ///     may be horribly inefficient.
    fn process_events(&self) -> Receiver<IncomingMessage> {
        println!("CliAdapter: process_events");
        // hmm.. there doesn't appear to be any way to select on stdin. Use a thread
        // until a better solution presents itself.
        let (tx_stdin, rx_stdin) = channel();
        thread::Builder::new().name("Chatbot CLI Reader".to_owned()).spawn(move || {

            loop {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    Ok(len) => {
                        if len == 0 {
                            break;
                        }
                        tx_stdin.send(line).unwrap();
                    },
                    Err(e) => {
                        println!("{:?}", e);
                        break;
                    }
                };
            }

            println!("CliAdapter: shutting down");
        }).ok().expect("failed to create stdio reader");

        let (tx_incoming, rx_incoming) = channel();
        let (tx_outgoing, rx_outgoing) = channel();
        let name = self.get_name().to_owned();

        thread::Builder::new().name("Chatbot CLI".to_owned()).spawn(move || {
            let select = Select::new();
            let mut outgoing = select.handle(&rx_outgoing);
            unsafe { outgoing.add() };
            let mut incoming = select.handle(&rx_stdin);
            unsafe { incoming.add() };

            loop {
                let id = select.wait();
                if id == outgoing.id() {
                    match rx_outgoing.recv().unwrap() {
                        AdapterMsg::Outgoing(msg) => {
                            io::stdout().write(msg.as_bytes()).unwrap();
                            io::stdout().write(b"\n").unwrap();
                            io::stdout().flush().unwrap();
                        },
                        _ => break
                    };
                } else if id == incoming.id() {
                    let bytes = rx_stdin.recv().unwrap();
                    let msg = IncomingMessage::new(name.to_owned(), None, None, None, bytes,
                        tx_outgoing.to_owned());
                    tx_incoming.send(msg).unwrap();
                }
            }
        }).ok().expect("failed to create stdio <-> chatbot proxy");

        rx_incoming
    }
}

