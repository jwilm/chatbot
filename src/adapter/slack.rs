extern crate slack;

use std::env;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread;

use rustc_serialize::json::Json;
use rustc_serialize::json;
use slack::Message;

use message::AdapterMsg;
use message::IncomingMessage;
use adapter::ChatAdapter;

/// SlackAdapter sends and receives messages from the Slack chat service. Until actualy
/// configuration is added, the slack token should be placed in the environment variable
/// `SLACK_BOT_TOKEN`
pub struct SlackAdapter;

impl SlackAdapter {
    pub fn new() -> SlackAdapter {
        SlackAdapter
    }
}

struct MyHandler {
  count: i64,
  tx_bot: Sender<IncomingMessage>,
  tx_adapter: Sender<AdapterMsg>
}

#[allow(unused_variables)]
impl slack::MessageHandler for MyHandler {
    fn on_receive(&mut self, cli: &mut slack::RtmClient, json_str: &str) {
        println!("Received[{}]: {}", self.count, json_str.to_string());
        self.count = self.count + 1;
        let json_thing = Json::from_str(json_str).unwrap();
        let json = json_thing.as_object().unwrap();

        let msg_type = match json.get("type") {
            Some(json_str) => json_str.as_string().unwrap(),
            None => return
        };

        match msg_type.as_ref() {
            "message" => {
                // TODO custom types for incoming messages.. unwrapunwrapunwrapunwrap
                let channel = json.get("channel").unwrap().as_string().unwrap();
                let user = json.get("user").unwrap().as_string().unwrap();
                let payload = json.get("text").unwrap().as_string().unwrap();

                let msg = IncomingMessage::new("SlackAdapter".to_owned(), None,
                    Some(channel.to_owned()), Some(user.to_owned()), payload.to_owned(),
                    self.tx_adapter.clone());

                self.tx_bot.send(msg).unwrap();
            },
            _ => {
                return ();
            }
        }
    }

    fn on_ping(&mut self, cli: &mut slack::RtmClient) {
        println!("<on_ping>");
    }

    fn on_close(&mut self, cli: &mut slack::RtmClient) {
        println!("<on_close>");
    }

    fn on_connect(&mut self, cli: &mut slack::RtmClient) {
        println!("<on_connect>");
        // let _ = cli.send_message("#general", "bla");
    }
}

impl ChatAdapter for SlackAdapter {
    /// SlackAdapter name
    fn get_name(&self) -> &str {
        "SlackAdapter"
    }

    fn process_events(&self) -> Receiver<IncomingMessage> {
        println!("SlackAdapter: process_events");
        let (tx_bot, rx_bot) = channel();
        let (tx_adapter, rx_adapter) = channel();

        let token = match env::var("SLACK_BOT_TOKEN") {
            Ok(t) => t,
            Err(_) => panic!("Failed to get SLACK_BOT_TOKEN from env")
        };

        let uid = AtomicIsize::new(0);

        let mut cli = slack::RtmClient::new();
        let (client, slack_rx) = cli.login(token.as_ref()).unwrap();
        let slack_tx = cli.get_outs().unwrap();

        thread::Builder::new().name("Chatbot Slack Receiver".to_owned()).spawn(move || {
            let mut handler = MyHandler{count: 0, tx_bot: tx_bot, tx_adapter: tx_adapter};
            cli.run::<MyHandler>(&mut handler, client, slack_rx).unwrap();
        }).ok().expect("failed to create thread for slack receiver");

        thread::Builder::new().name("Chatbot Slack Sender".to_owned()).spawn(move || {
            loop {
                match rx_adapter.recv() {
                    Ok(msg) => {
                        match msg {
                            AdapterMsg::Outgoing(msg) => {
                                let n = uid.fetch_add(1, Ordering::SeqCst);
                                let chan = msg.get_incoming().channel().unwrap();
                                let payload = msg.as_ref();
                                let mut obj = json::Object::new();
                                obj.insert("id".to_string(), Json::I64(n as i64));
                                obj.insert("channel".to_string(), Json::String(chan.to_owned()));
                                obj.insert("type".to_string(), Json::String("message".to_owned()));
                                obj.insert("text".to_string(), Json::String(payload.to_owned()));
                                let json = Json::Object(obj);
                                let slack_msg = Message::Text(json.to_string());
                                slack_tx.send(slack_msg).unwrap();
                            }
                            _ => println!("TODO")
                        }
                    },
                    Err(e) => {
                        println!("error receiving outgoing messages to the slack adapter: {}", e);
                        break
                    }
                }
            }
        }).ok().expect("failed to create thread for slack sender");

        rx_bot
    }
}

