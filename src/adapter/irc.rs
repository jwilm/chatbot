extern crate irc;

use std::sync::Mutex;
use std::sync::Arc;
use std::thread;

/// Reexport the irc crate server config for now. Example programs can just grab it from here rather
/// than know the irc crate internals.
pub use irc::client::data::Config;

use irc::client::data::Command;
use irc::client::data::message::ToMessage;
use irc::client::server::IrcServer;
use irc::client::server::Server;
use irc::client::server::utils::ServerExt;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use chatbot::Chatbot;
use adapter::ChatAdapter;
use message::IncomingMessage;

/// Connect your bot to IRC with the IrcAdapter
pub struct IrcAdapter {
    config: Config
}

impl IrcAdapter {
    pub fn new(config: Config) -> IrcAdapter {
        IrcAdapter {
            config: config
        }
    }
}

impl ChatAdapter for IrcAdapter {
    fn get_name(&self) -> &str {
        "IrcAdapter"
    }

    fn process_events(&self, bot: &Chatbot, tx_incoming: Sender<IncomingMessage>) {
        let server = Arc::new(IrcServer::from_config(self.config.clone()).unwrap());
        server.identify().unwrap();

        let (tx_outgoing, rx_outgoing) = channel();
        let name = bot.get_name().to_owned();

        {
            let server = server.clone();
            thread::Builder::new().name("IrcAdapter lisener".to_owned()).spawn(move || {

                for message in server.iter() {
                    if message.is_err() {
                        continue;
                    }

                    let msg = message.unwrap();

                    let user = msg.get_source_nickname().map(|user| user.to_owned());
                    if let Ok(command) = Command::from_message(&msg) {
                        match command {
                            Command::PRIVMSG(ref chan, ref msg) => {
                                let incoming = IncomingMessage::new("IrcAdapter".to_owned(),
                                    Some(server.config().server().to_owned()),
                                    Some(chan.to_owned()), user, msg.to_owned(),
                                    tx_outgoing.clone());

                                println!("{:?}", incoming);

                                tx_incoming.send(incoming)
                                           .ok().expect("chatbot not receiving messages");
                            },
                            _ => ()
                        }
                    }
                }
            }).ok().expect("failed to create irc listener thread");
        }
    }
}
