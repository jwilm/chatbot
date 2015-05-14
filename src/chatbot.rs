use adapter::ChatAdapter;
use handler::MessageHandler;
use std::collections::HashMap;
use std::sync::mpsc::Select;

pub struct Chatbot {
    name: String,
    adapters: Vec<Box<ChatAdapter>>,
    handlers: Vec<Box<MessageHandler>>
}

impl Chatbot {
    pub fn new() -> Chatbot {
        Chatbot {
            name: "computer".to_owned(),
            adapters: Vec::new(),
            handlers: Vec::new()
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn add_adapter<T: ChatAdapter + 'static>(&mut self, adapter: Box<T>) {
        println!("Adding adapter {}", adapter.get_name());
        self.adapters.push(adapter)
    }

    pub fn add_handler<T: MessageHandler + 'static>(&mut self, handler: Box<T>) {
        println!("Adding handler {}", handler.get_name());
        self.handlers.push(handler)
    }

    pub fn run(&self) {

        let select = Select::new();
        // Use two maps here. One maps adapter names to channel ports, the other
        // maps handle ids to adapter names.
        let mut handles = HashMap::new();
        let mut channels = HashMap::new();

        // TODO this could be cleaned up if more information could be stored on
        // the adapter.
        println!("Chatbot: starting {} adapters", self.adapters.len());
        for adapter in &self.adapters {
            println!("Chatbot: starting adapter {}", adapter.get_name());
            // store ports in channels
            channels.insert(adapter.get_name(), adapter.process_events());
            // get ref to ports
            let &(ref send, ref recv) = channels.get(adapter.get_name()).unwrap();
            // set up select handle
            let mut handle = select.handle(recv);
            unsafe { handle.add() };
            // map handle to adapter
            handles.insert(handle.id(), adapter.get_name());
        };

        println!("Chatbot: entering main loop");
        loop {
            println!("Chatbot: select");
            let id = select.wait();
            println!("Chatbot: done waiting");
            let &(ref sender, ref receiver) = channels.get(handles.get(&id).unwrap()).unwrap();

            match receiver.recv().unwrap() {
                _ => break
            }
        }
        println!("Chatbot shutting down");
    }
}

#[cfg(test)]
mod tests {
    use chatbot::Chatbot;
    use adapter::CliAdapter;
    use handler::EchoHandler;

    #[test]
    fn test_create_chatbot() {
        let bot = Chatbot::new();
        assert_eq!(bot.get_name(), "computer");
    }

    #[test]
    fn test_chatbot_add_adapter() {
        let mut bot = Chatbot::new();
        let cli = Box::new(CliAdapter::new());
        bot.add_adapter(cli);
    }

    #[test]
    fn test_chatbot_add_handler() {
        let mut bot = Chatbot::new();
        let handler = Box::new(EchoHandler::new());
        bot.add_handler(handler);
    }
}
