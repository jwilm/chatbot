pub trait MessageHandler {
    fn get_name(&self) -> &str;
    fn on_message(&self, payload: &IncomingMessage) -> Option<OutgoingMessage>;
}

pub struct IncomingMessage {
    message: String,
    from_adapter: String,
    server: Option<String>,
    channel: Option<String>,
    user: Option<String>,
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
}

impl IncomingMessage {
    pub fn new(from_adapter: String, server: String, channel: String, user: String,
               message: String) -> IncomingMessage {
        IncomingMessage {
            from_adapter: from_adapter,
            server: server,
            channel: channel,
            user: user,
            message: message
        }
    }

    pub fn get_contents(&self) -> &str {
        self.message.as_ref()
    }
}

