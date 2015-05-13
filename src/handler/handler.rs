struct IncomingMessage;
struct OutgoingMessage;

pub trait MessageHandler {
    fn get_name(&self) -> &str;
    fn handler(&self, payload: IncomingMessage) -> Option<OutgoingMessage>;
}
