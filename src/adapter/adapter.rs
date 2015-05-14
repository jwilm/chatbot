pub trait ChatAdapter {
    fn get_name(&self) -> &str;
    fn process_events(&self) -> (Receiver<IncomingMessage>, Sender<OutgoingMessage>);
}
