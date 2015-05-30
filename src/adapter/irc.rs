extern crate irc;

use std::sync::Arc;
use std::thread;

pub type IrcConfig = irc::client::data::Config;

use irc::client::data::Command;
use irc::client::server::IrcServer;
use irc::client::server::Server;
use irc::client::server::utils::ServerExt;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use chatbot::Chatbot;
use adapter::ChatAdapter;
use message::IncomingMessage;
use message::AdapterMsg;

/// Connect your bot to IRC with the IrcAdapter
///
/// # Examples
///
/// ```rust
/// use chatbot::Chatbot;
/// use chatbot::adapter::IrcAdapter;
/// use chatbot::adapter::IrcConfig;
///
/// let name = "mybot";
/// let mut bot = Chatbot::new(name);
///
/// let irc = IrcAdapter::new(IrcConfig {
///     nickname: Some(format!("{}", name)),
///     alt_nicks: Some(vec![format!("{}_", name), format!("{}__", name)]),
///     server: Some(format!("irc.mozilla.org")),
///     channels: Some(vec![format!("#chatbot")]),
///     .. Default::default()
/// });
///
/// bot.add_adapter(irc);
/// ```
pub struct IrcAdapter {
    config: IrcConfig
}

impl IrcAdapter {
    pub fn new(config: IrcConfig) -> IrcAdapter {
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
            thread::Builder::new().name("IrcAdapter Incoming".to_owned()).spawn(move || {

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

                                tx_incoming.send(incoming)
                                           .ok().expect("chatbot not receiving messages");
                            },
                            _ => ()
                        }
                    }
                }
            }).ok().expect("failed to create incoming thread for IrcAdapter");
        }

        thread::Builder::new().name("IrcAdapter Outgoing".to_owned()).spawn(move || {
            loop {
                match rx_outgoing.recv() {
                    Ok(msg) => {
                        match msg {
                            AdapterMsg::Outgoing(m) => {
                                let incoming = m.get_incoming();
                                let chan = incoming.channel().unwrap();
                                let user = incoming.user().unwrap();

                                if &name[..] == chan {
                                    server.send_privmsg(user, m.as_ref()).unwrap()
                                } else {
                                    server.send_privmsg(chan, m.as_ref()).unwrap()
                                };

                            }
                            _ => unreachable!("No other messages being sent yet")
                        }
                    },
                    Err(e) => {
                        println!("error receiving outgoing messages: {}", e);
                        break
                    }
                }
            }
        }).ok().expect("failed to create outgoing thread for IrcAdapter");
    }
}
