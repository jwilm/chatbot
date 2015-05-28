use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use rustc_serialize::json::{self, Json, ToJson};

use message::OutgoingMessage;

/// Data for an Event::Message(Msg::Plain)
#[allow(dead_code)]
pub struct MessageData {
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
    /// Get the message text
    str_accessor!(text);
    /// Channel where the message was sent
    str_accessor!(channel);
    /// User who sent the message
    str_accessor!(user);
    /// When the message was sent
    str_accessor!(ts);
    /// Team owning the message
    str_accessor!(team);
}

/// Incoming slack messages on the websocket api
pub enum Event {
    /// A message was sent to a channel
    Message(Msg),

    /// Some other type of message arrived.
    /// [The list](https://api.slack.com/rtm) is quite extensive, and only the
    /// messages the adapter is concerned with are enumerated here.
    Other(json::Object)
}

/// Event::Message sub types. Message events are the only event capable of having a sub type.
pub enum Msg {
    /// A regular text message from a user
    Plain(MessageData),

    /// Subtype not explicitly handled. For a complete enumeration, check
    /// https://api.slack.com/events/message
    Other(json::Object)
}

/// Errors that can occur when parsing a slack JSON message
#[derive(Debug)]
pub enum EventDecodingError {
    InvalidJson(json::BuilderError),
    MissingField(String),
    WrongType(String, String),
}

impl Error for EventDecodingError {
    fn description(&self) -> &str {
        match *self {
            EventDecodingError::InvalidJson(ref err) => err.description(),
            EventDecodingError::MissingField(_) => "JSON property missing",
            EventDecodingError::WrongType(_, _) => "JSON property had wrong type"
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            EventDecodingError::InvalidJson(ref err) => Some(err),
            _ => None
        }
    }
}

impl fmt::Display for EventDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EventDecodingError::InvalidJson(ref err) => write!(f, "InvalidJson({})", err),
            EventDecodingError::MissingField(ref field) => write!(f, "MissingField({})", field),
            EventDecodingError::WrongType(ref field, ref t) => {
                write!(f, "WrongType({}, {})", field, t)
            }
        }
    }
}

impl From<json::BuilderError> for EventDecodingError {
    fn from(err: json::BuilderError) -> EventDecodingError {
        EventDecodingError::InvalidJson(err)
    }
}

/// Extract a &str ($key) from a json::Object ($obj)
/// Returns an Err(EventDecodingError) when it fails
macro_rules! get_json_string {
    ($obj:ident, $key:expr) => {
        {
            let json_str: &Json = match $obj.get($key) {
                Some(json_str) => json_str,
                None => return Err(EventDecodingError::MissingField($key.to_owned()))
            };

            match json_str.as_string() {
                Some(slice) => slice.to_owned(),
                None => {
                    return Err(EventDecodingError::WrongType($key.to_owned(), "string".to_owned()))
               }
            }
        }
    }
}

/// Decode the Event::Message data
fn decode_msg_json(obj: json::Object) -> Result<Event, EventDecodingError> {
    // Messages with a `subtype` are not plain text messages.. Not interested in them for now.
    if obj.contains_key("subtype") {
        return Ok(Event::Message(Msg::Other(obj)))
    }

    // It's a plain message.
    Ok(Event::Message(Msg::Plain(MessageData {
        text: get_json_string!(obj, "text"),
        channel: get_json_string!(obj, "channel"),
        user: get_json_string!(obj, "user"),
        ts: get_json_string!(obj, "ts"),
        team: get_json_string!(obj, "team"),
    })))
}

/// Convert a JSON string to a Event
pub fn string_to_slack_msg(raw: &str) -> Result<Event, EventDecodingError> {
    // The message should be a string representation of a JSON object
    let json = try!(Json::from_str(raw));
    let obj = match json {
        Json::Object(obj) => obj,
        _ => return Err(EventDecodingError::WrongType("root".to_owned(), "object".to_owned()))
    };

    // If the message does not have a `type`, it is a confirmation message
    if !obj.contains_key("type") {
        return Ok(Event::Other(obj))
    }

    // Get the message type. It should be a string.
    let msg = match obj.get("type").expect("obj has key type but failed to get").as_string() {
        Some(s) => s.to_owned(),
        None => return Err(EventDecodingError::WrongType("type".to_owned(), "string".to_owned()))
    };

    // Handle different message types
    match msg.as_ref() {
        // If it's a message type, try and return an Event::Message
        "message" => decode_msg_json(obj),
        // Some other type we don't explicitly handle
        _ => Ok(Event::Other(obj)),
    }
}

/// An outgoing slack event.
///
/// The OutgoingEvent type only supports type = "message" at this time. The `type` field is recorded
/// in the struct as `msg_type` since rust has a conflicting `type` keyword.
#[derive(Debug)]
pub struct OutgoingEvent {
    id: i64,
    channel: String,
    msg_type: String,
    text: String
}

impl OutgoingEvent {
    pub fn new(id: i64, m: OutgoingMessage) -> OutgoingEvent {
        OutgoingEvent {
            id: id,
            channel: m.get_incoming().channel().expect("missing channel").to_owned(),
            msg_type: "message".to_owned(),
            text: m.as_ref().to_owned() // TODO move instead of copy
        }
    }
}

impl ToJson for OutgoingEvent {
    fn to_json(&self) -> Json {
        let mut d = BTreeMap::new();
        d.insert("id".to_string(), self.id.to_json());
        d.insert("channel".to_string(), self.channel.to_json());
        d.insert("type".to_string(), self.msg_type.to_json());
        d.insert("text".to_string(), self.text.to_json());
        Json::Object(d)
    }
}

#[cfg(test)]
mod tests {
    use adapter::slack::message::Event;
    use adapter::slack::message::Msg;
    use adapter::slack::message::string_to_slack_msg;

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
            Event::Message(Msg::Plain(data)) => {
                assert_eq!(data.text(), "ping");
                assert_eq!(data.channel(), "D04UYUAMW");
                assert_eq!(data.user(), "U02ALMR84");
                assert_eq!(data.ts(), "1432563914.000007");
                assert_eq!(data.team(), "T02ALMR82");
            },
            _ => panic!("Expected Event::Message")
        }
    }
    #[test]
    fn decode_confirmation_message() {
        let raw = r#"{"ok":true,"reply_to":0,"ts":"1432566639.000014","text":"pong"}"#;

        match string_to_slack_msg(raw).unwrap() {
            Event::Other(_) => (),
            _ => panic!("expected Event::Other")
        }
    }

    #[test]
    fn decode_unexpected_type() {
        let raw = r#"{"type":"not_a_slack_msg_type"}"#;

        match string_to_slack_msg(raw).unwrap() {
            Event::Other(_) => (),
            _ => panic!("expected Event::Other")
        }
    }

    #[test]
    fn decode_message_changed() {
        let raw = r#"{
            "type":"message",
            "message":{
                "type":"message",
                "user":"U02ALMR84",
                "text":"arst",
                "edited":{
                    "user":"U02ALMR84",
                    "ts":"1432695814.000000"
                },
                "ts":"1432695812.000056"
            },
            "subtype":"message_changed",
            "hidden":true,
            "channel":"D04UYUAMW",
            "event_ts":"1432695814.616510",
            "ts":"1432695814.000057"
        }"#;

        match string_to_slack_msg(raw).unwrap() {
            Event::Message(Msg::Other(_)) => return,
            _ => panic!("expected Msg::Other")
        }
    }

    #[test]
    fn decode_me_message() {
        let raw = r#"{
            "text":"is a potato",
            "type":"message",
            "subtype":"me_message",
            "user":"U02ALMR84",
            "channel":"D04UYUAMW",
            "ts":"1432695826.000060"
        }"#;

        match string_to_slack_msg(raw).unwrap() {
            Event::Message(Msg::Other(_)) => return,
            _ => panic!("expected Msg::Other")
        }
    }

    #[test]
    fn decode_message_deleted() {
        let raw = r#"{
            "type":"message",
            "deleted_ts":"1432695826.000060",
            "subtype":"message_deleted",
            "hidden":true,
            "channel":"D04UYUAMW",
            "event_ts":"1432695848.617155",
            "ts":"1432695848.000061"
        }"#;

        match string_to_slack_msg(raw).unwrap() {
            Event::Message(Msg::Other(_)) => return,
            _ => panic!("expected Msg::Other")
        }
    }

}
