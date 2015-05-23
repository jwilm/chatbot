use adapter::ChatAdapter;
use handler::MessageHandler;

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
        // Temporarily limit the number of concurrent adapters to 1 until
        // switching to mio or similar
        assert_eq!(self.adapters.len(), 0);
        self.adapters.push(adapter)
    }

    pub fn add_handler<T: MessageHandler + 'static>(&mut self, handler: Box<T>) {
        println!("Adding handler {}", handler.name());
        self.handlers.push(handler)
    }

    pub fn run(&self) {

        println!("Chatbot: starting {} adapters", self.adapters.len());

        let ref adapter = self.adapters[0];
        let adapter_rx = adapter.process_events();

        println!("Have {} receivers", self.adapters.len());
        println!("Chatbot: entering main loop");

        loop {
            // Get message from adapter
            let msg = match adapter_rx.recv() {
                Ok(msg) => msg,
                Err(_) => break
            };

            // Distribute to handlers
            for handler in &self.handlers {
                if let Err(e) = handler.handle(&msg) {
                    // TODO check error variant and release adapters as needed.
                    println!("Error in handler.handle: {:?}", e);
                    break;
                }
            }
        }

        println!("chatbot shutting down");
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
