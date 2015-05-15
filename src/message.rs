use std::sync::mpsc::Sender;

use adapter::AdapterMsg;

pub struct IncomingMessage {
    message: String,
    from_adapter: String,
    server: Option<String>,
    channel: Option<String>,
    user: Option<String>,
    tx: Sender<AdapterMsg>
}

pub struct OutgoingMessage {
    response: String
}

impl OutgoingMessage {
    pub fn new(response: String) -> OutgoingMessage {
        OutgoingMessage {
            response: response
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.response.as_bytes()
    }
}

impl IncomingMessage {
    pub fn new(from_adapter: String, server: Option<String>, channel: Option<String>,
               user: Option<String>, message: String,
               sender: Sender<AdapterMsg>) -> IncomingMessage {
        IncomingMessage {
            from_adapter: from_adapter,
            server: server,
            channel: channel,
            user: user,
            message: message,
            tx: sender
        }
    }

    pub fn get_contents(&self) -> &str {
        self.message.as_ref()
    }

    pub fn reply(&self, msg: AdapterMsg) {
        self.tx.send(msg);
    }
}

