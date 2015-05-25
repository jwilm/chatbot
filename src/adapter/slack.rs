extern crate slack;

use std::collections::BTreeMap;
use std::env;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread;

use rustc_serialize::json::{self, Json, ToJson};
use rustc_serialize::json::DecoderError::MissingFieldError;
use rustc_serialize::Decodable;
use rustc_serialize::Decoder;

use slack::Message;

use adapter::ChatAdapter;
use message::AdapterMsg;
use message::IncomingMessage;
use message::OutgoingMessage;

/// SlackAdapter sends and receives messages from the Slack chat service. Until actualy
/// configuration is added, the slack token should be placed in the environment variable
/// `SLACK_BOT_TOKEN`
pub struct SlackAdapter {
    token: String
}

impl SlackAdapter {
    pub fn new() -> SlackAdapter {
        SlackAdapter {
            token: match env::var("SLACK_BOT_TOKEN") {
                Ok(t) => t,
                Err(_) => panic!("Failed to get SLACK_BOT_TOKEN from env")
            }
        }
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

/// Convert a JSON string to a SlackMsg
///
/// This methods provides additional error handling around json::decode for certain errors
/// that cannot be handled in the Decodable implementation. Specifically, MissingFieldError
/// where the field is "type" are actually valid messages despite missing the "type" field.
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

#[derive(Debug)]
struct OutgoingSlackMsg {
    id: i64,
    channel: String,
    msg_type: String,
    text: String
}

impl OutgoingSlackMsg {
    fn new(id: i64, m: OutgoingMessage) -> OutgoingSlackMsg {
        OutgoingSlackMsg {
            id: id,
            channel: m.get_incoming()
                      .channel()
                      .expect("missing channel").to_owned(),
            msg_type: "message".to_owned(),
            text: m.as_ref().to_owned() // TODO move instead of copy
        }
    }
}

impl ToJson for OutgoingSlackMsg {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("id".to_string(), self.id.to_json());
        d.insert("channel".to_string(), self.channel.to_json());
        d.insert("type".to_string(), self.msg_type.to_json());
        d.insert("text".to_string(), self.text.to_json());
        Json::Object(d)
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

    fn on_ping(&mut self, cli: &mut slack::RtmClient) { }

    fn on_close(&mut self, cli: &mut slack::RtmClient) { }

    fn on_connect(&mut self, cli: &mut slack::RtmClient) { }
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

        let uid = AtomicIsize::new(0);

        let mut cli = slack::RtmClient::new();
        let (client, slack_rx) = cli.login(self.token.as_ref()).unwrap();
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
                            AdapterMsg::Outgoing(m) => {
                                let id = uid.fetch_add(1, Ordering::SeqCst) as i64;
                                let out = OutgoingSlackMsg::new(id, m);
                                slack_tx.send(Message::Text(out.to_json().to_string())).unwrap();
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
