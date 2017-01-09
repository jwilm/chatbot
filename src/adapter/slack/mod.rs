mod message;
use self::message::*;

use std::collections::HashMap;
use std::env;
use std::sync::mpsc::{self, Sender, channel};
use std::thread;

use rustc_serialize::json::ToJson;

use slack;
use regex::Regex;

use adapter::ChatAdapter;
use message::AdapterMsg;
use message::IncomingMessage;

/// SlackAdapter sends and receives messages from the Slack chat service. Until actualy
/// configuration is added, the slack token should be placed in the environment variable
/// `SLACK_BOT_TOKEN`
pub struct SlackAdapter {
    client: Option<slack::RtmClient>,
    login_state: Option<(slack::WsClient, mpsc::Receiver<slack::WsMessage>)>,
    addresser_regex: Regex,
}

impl SlackAdapter {
    pub fn new() -> SlackAdapter {
        let token = env::var("SLACK_BOT_TOKEN").expect("Failed to get SLACK_BOT_TOKEN from env");

        let mut cli = slack::RtmClient::new(&token[..]);
        let login_state = cli.login().expect("login to slack");

        let id = cli.get_id().unwrap();
        let addresser_regex = Regex::new(format!(r"^<@{}>", id).as_str()).unwrap();

        SlackAdapter {
            client: Some(cli),
            login_state: Some(login_state),
            addresser_regex: addresser_regex,
        }
    }
}

struct MyHandler {
  count: i64,
  tx_incoming: Sender<IncomingMessage>,
  tx_outgoing: Sender<AdapterMsg>,
  users: HashMap<String, slack::User>,
}

#[allow(unused_variables)]
impl slack::EventHandler for MyHandler {
    fn on_event(&mut self,
                cli: &mut slack::RtmClient,
                event: Result<&slack::Event, slack::Error>,
                raw: &str)
    {
        println!("Received[{}]: {}", self.count, raw.to_string());
        self.count = self.count + 1;

        match string_to_slack_msg(raw) {
            Ok(slack_msg) => {
                match slack_msg {
                    Event::Message(Msg::Plain(msg)) => {
                        let user = self.users.get(msg.user())
                                             .map(|u| u.name.clone())
                                             .unwrap_or_else(|| msg.user().to_owned());

                        let incoming = IncomingMessage::new("SlackAdapter".to_owned(), None,
                            Some(msg.channel().to_owned()), Some(user),
                            msg.text().to_owned(), self.tx_outgoing.clone());

                        self.tx_incoming.send(incoming)
                                        .ok().expect("Bot unable to process messages");
                    },
                    _ => ()
                }
            },
            Err(e) => {
                println!("error decoding slack message: {:?}", e);
                println!("please consider reporting this to jwilm/chatbot as it is probably a bug");
            }
        }
    }

    fn on_ping(&mut self, cli: &mut slack::RtmClient) { }

    fn on_close(&mut self, cli: &mut slack::RtmClient) { }

    fn on_connect(&mut self, cli: &mut slack::RtmClient) { }
}

impl ChatAdapter for SlackAdapter {
    /// SlackAdapter name
    fn get_name(&self) -> &str {
        "SlackAdapter"
    }

    /// Check whether this adapter was addressed
    fn addresser(&self) -> &Regex {
        &self.addresser_regex
    }

    fn process_events(&mut self, tx_incoming: Sender<IncomingMessage>) {
        println!("SlackAdapter: process_events");
        let (tx_outgoing, rx_outgoing) = channel();

        let mut cli = self.client.take().unwrap();
        let (client, slack_rx) = self.login_state.take().unwrap();

        let slack_tx = cli.channel().expect("get a slack sender");

        thread::Builder::new().name("Chatbot Slack Receiver".to_owned()).spawn(move || {
            let users = cli.list_users().expect("get users list")
                           .into_iter()
                           .map(|u| (u.id.clone(), u))
                           .collect();
            let mut handler = MyHandler {
                count: 0,
                tx_incoming: tx_incoming,
                tx_outgoing: tx_outgoing,
                users: users,
            };
            cli.run(&mut handler, client, slack_rx).expect("run connector ok");
        }).ok().expect("failed to create thread for slack receiver");

        thread::Builder::new().name("Chatbot Slack Sender".to_owned()).spawn(move || {
            loop {
                match rx_outgoing.recv() {
                    Ok(msg) => {
                        match msg {
                            AdapterMsg::Outgoing(m) => {
                                let id = slack_tx.get_msg_uid() as i64;
                                let out = OutgoingEvent::new(id, m);
                                slack_tx.send(out.to_json().to_string().as_ref())
                                        .expect("send message ok");
                            }
                            // Not implemented for now
                            AdapterMsg::Private(_) => {
                                println!("SlackAdaptor: Private messages not implemented");
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
        }).ok().expect("failed to create thread for slack sender");
    }
}
