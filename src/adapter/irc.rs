use std::thread;

pub type IrcConfig = ::irc::client::data::Config;

use irc::proto::command::Command;
use irc::client::server::IrcServer;
use irc::client::server::Server;
use irc::client::server::utils::ServerExt;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use regex::Regex;

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
/// }, name);
///
/// bot.add_adapter(irc);
/// ```
pub struct IrcAdapter {
    config: IrcConfig,
    address_regex: Regex,
    name: String,
}

impl IrcAdapter {
    pub fn new(config: IrcConfig, bot_name: &str) -> IrcAdapter {
        IrcAdapter {
            config: config,
            address_regex: Regex::new(format!(r"^{}:", bot_name).as_str()).unwrap(),
            name: bot_name.to_owned(),
        }
    }
}

impl ChatAdapter for IrcAdapter {
    fn get_name(&self) -> &str {
        "IrcAdapter"
    }

    fn addresser(&self) -> &Regex {
        &self.address_regex
    }

    fn process_events(&mut self, tx_incoming: Sender<IncomingMessage>) {
        let server = IrcServer::from_config(self.config.clone()).unwrap();
        server.identify().unwrap();

        let (tx_outgoing, rx_outgoing) = channel();
        let name = self.name.clone();

        {
            let server = server.clone();
            thread::Builder::new().name("IrcAdapter Incoming".to_owned()).spawn(move || {
                server.for_each_incoming(|message| {
                    let user = message.source_nickname().map(|user| user.to_owned());
                    match message.command {
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
                }).expect("error processing IrcServer::for_each_incoming");
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
                            AdapterMsg::Private(m) => {
                                let incoming = m.get_incoming();
                                let user = incoming.user().unwrap();
                                server.send_privmsg(user, m.as_ref()).unwrap()
                            }
                            AdapterMsg::Shutdown => {
                                break
                            }
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
