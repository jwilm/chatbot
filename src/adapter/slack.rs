extern crate slack;

use std::env;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread;

use rustc_serialize::json::Json;
use rustc_serialize::json;
use rustc_serialize::json::DecoderError::MissingFieldError;
use rustc_serialize::Decodable;
use rustc_serialize::Decoder;

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

/// Data for a SlackMsg::Message
#[allow(dead_code)]
struct MessageData {
    text: String,
    channel: String,
    user: String,
    ts: String,
    team: String
}

macro_rules! str_accessor {
    ($s:ident) => {
        pub fn $s(&self) -> &str {
            self.$s.as_ref()
        }
    }
}

#[allow(dead_code)]
impl MessageData {
    str_accessor!(text);
    str_accessor!(channel);
    str_accessor!(user);
    str_accessor!(ts);
    str_accessor!(team);
}

/// Incoming slack messages on the websocket api
enum SlackMsg {
    /// A message was sent to a channel
    Message(MessageData),

    /// Some other type of message arrived.
    /// [The list](https://api.slack.com/rtm) is quite extensive, and only the
    /// messages the adapter is concerned with are enumerated here.
    Other
}

impl Decodable for SlackMsg {
    fn decode<D: Decoder>(d: &mut D) -> Result<SlackMsg, D::Error> {
        d.read_struct("root", 0, |root| {
            let msg_type = try!(root.read_struct_field("type", 0, |f| f.read_str() ));

            match msg_type.as_ref() {
                "message" => {
                    Ok(SlackMsg::Message(MessageData {
                        text: try!(root.read_struct_field("text", 0, |f| f.read_str())),
                        channel: try!(root.read_struct_field("channel", 0, |f| f.read_str())),
                        user: try!(root.read_struct_field("user", 0, |f| f.read_str())),
                        ts: try!(root.read_struct_field("ts", 0, |f| f.read_str())),
                        team: try!(root.read_struct_field("team", 0, |f| f.read_str())),
                    }))
                },
                _ => {
                    Ok(SlackMsg::Other)
                }
            }
        })
    }
}

fn string_to_slack_msg(raw: &str) -> Result<SlackMsg, json::DecoderError> {
    // Some messages arriving from the slack client don't have a type. So far I've only
    // witnessed confirmation messages arriving in this fashion. Since they go through the same
    // pipeline as content messages, the decoder should be able to handle them.
    match json::decode::<SlackMsg>(raw) {
        Ok(value) => Ok(value),
        Err(e) => {
            match e {
                MissingFieldError(field) => {
                    match field.as_ref() {
                        "type" => Ok(SlackMsg::Other),
                        _ => Err(MissingFieldError(field))
                    }
                },
                _ => Err(e)
            }
        }
    }
}

#[allow(unused_variables)]
impl slack::MessageHandler for MyHandler {
    fn on_receive(&mut self, cli: &mut slack::RtmClient, raw: &str) {
        println!("Received[{}]: {}", self.count, raw.to_string());
        self.count = self.count + 1;

        match string_to_slack_msg(raw) {
            Ok(slack_msg) => {
                match slack_msg {
                    SlackMsg::Message(msg) => {
                        let incoming = IncomingMessage::new("SlackAdapter".to_owned(), None,
                            Some(msg.channel().to_owned()), Some(msg.user().to_owned()),
                            msg.text().to_owned(), self.tx_adapter.clone());

                        self.tx_bot.send(incoming).ok().expect("Bot unable to process messages");
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

#[cfg(test)]
mod tests {
    use adapter::slack::SlackMsg;
    use adapter::slack::string_to_slack_msg;

    #[test]
    fn decode_message() {
        let raw = "{\"type\":\"message\", \
                    \"channel\":\"D04UYUAMW\", \
                    \"user\":\"U02ALMR84\", \
                    \"text\":\"ping\", \
                    \"ts\":\"1432563914.000007\", \
                    \"team\":\"T02ALMR82\"}";

        let slack_msg = string_to_slack_msg(raw).unwrap();

        match slack_msg {
            SlackMsg::Message(data) => {
                assert_eq!(data.text(), "ping");
                assert_eq!(data.channel(), "D04UYUAMW");
                assert_eq!(data.user(), "U02ALMR84");
                assert_eq!(data.ts(), "1432563914.000007");
                assert_eq!(data.team(), "T02ALMR82");
            },
            _ => panic!("Expected SlackMsg::Message")
        }
    }
    #[test]
    fn decode_confirmation_message() {
        let raw = r#"{"ok":true,"reply_to":0,"ts":"1432566639.000014","text":"pong"}"#;

        match string_to_slack_msg(raw).unwrap() {
            SlackMsg::Other => (),
            _ => panic!("expected SlackMsg::Other")
        }
    }

    #[test]
    fn decode_unexpected_type() {
        let raw = r#"{"type":"not_a_slack_msg_type"}"#;

        match string_to_slack_msg(raw).unwrap() {
            SlackMsg::Other => (),
            _ => panic!("expected SlackMsg::Other")
        }
    }
}
