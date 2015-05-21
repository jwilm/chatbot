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
        println!("Adding handler {}", handler.name());
        self.handlers.push(handler)
    }

    pub fn run(&self) {


        // TODO this could be cleaned up if more information could be stored on
        // the adapter.
        println!("Chatbot: starting {} adapters", self.adapters.len());

        let sel = Select::new();

        let receivers = self.adapters.iter().map(|adapter| {
            adapter.process_events()
        }).collect::<Vec<_>>();

        let mut handles = HashMap::new();
        for rx in &receivers {
            let handle = sel.handle(&rx);
            let id = handle.id();
            handles.insert(id, handle);
            let mut h = handles.get_mut(&id).unwrap();
            unsafe { (*h).add() };
        }

        println!("Have {} receivers", receivers.len());

        println!("Chatbot: entering main loop");

        loop {
            let id = sel.wait();
            let handle = handles.get_mut(&id).unwrap();

            let msg = match handle.recv() {
                Ok(msg) => msg,
                Err(e) => panic!("Chatbot main channel error {}", e)
            };

            println!("Got message: {}", msg.get_contents());
            for handler in &self.handlers {
                if let Err(e) = handler.handle(&msg) {
                    // TODO check error variant and release adapters as needed.
                    println!("Error in handler.handle: {:?}", e);
                    break;
                }
            }
        }

        // TODO there's a crash when this falls through
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
