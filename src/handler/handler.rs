pub trait MessageHandler {
    fn get_name(&self) -> &str;
    fn on_message<'a>(&'a self, payload: &'a IncomingMessage<'a>) -> Option<OutgoingMessage>;
}

pub struct IncomingMessage<'a> {
    from_adapter: &'a str,
    server: &'a str,
    channel: &'a str,
    user: &'a str,
    message: String
}

pub struct OutgoingMessage<'a> {
    incoming: &'a IncomingMessage<'a>,
    response: String
}

impl<'a> OutgoingMessage<'a> {
    pub fn new(incoming: &'a IncomingMessage, response: String) -> OutgoingMessage<'a> {
        OutgoingMessage {
            incoming: incoming,
            response: response
        }
    }
}

impl<'a> IncomingMessage<'a> {
    pub fn new(from_adapter: &'a str, server: &'a str, channel: &'a str, user: &'a str,
               message: String) -> IncomingMessage<'a> {
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

