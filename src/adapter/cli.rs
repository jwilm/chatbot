use std::sync::mpsc::Select;
use std::sync::mpsc::channel;
use std::thread;

use adapter::ChatAdapter;

pub struct CliAdapter {
    name: &'static str
}

impl CliAdapter {
    pub fn new() -> CliAdapter {
        CliAdapter {
            name: "cli"
        }
    }
}

impl ChatAdapter for CliAdapter {
    fn get_name(&self) -> &str {
        self.name
    }

    fn process_events(&self) -> (Receiver<IncomingMessage>, Sender<AdapterMsg>) {
        // hmm.. there doesn't appear to be any way to select on stdin. Use a thread
        // until a better solution presents itself.
        let (tx_stdin, rx_stdin) = channel();
        thread::Builder::new().name("Chatbot CLI Reader".to_owned()).spawn(move || {
            loop {
                let mut line = String::new();
                let len = io::stdin().read_line(&mut line).unwrap();

                if !len {
                    break;
                }

                tx_stdin.send(line);
            }
        });

        let (tx_incoming, rx_incoming) = channel();
        let (tx_outgoing, rx_outgoing) = channel();
        let name = self.name.to_owned();

        thread::Builder::new().name("Chatbot CLI".to_owned()).spawn(move || {
            let select = Select::new();
            let outgoing = select.handle(&rx_outgoing);
            unsafe { outgoing.add() };
            let incoming = select.handle(&rx_stdin);
            unsafe { incoming.add() };

            loop {
                let id = select.wait();
                if id == outgoing.id() {
                    let msg = rx_outgoing.recv().unwrap();
                    io::stdout().write(&msg).unwrap();
                } else if id == incoming.id() {
                    let msg = rx_stdin.recv().unwrap();
                    tx_incoming.send(IncomingMessage::new(name.clone(), None, None, None, msg));
                }
            }
        });

        (tx_outgoing, rx_incoming)
    }
}

