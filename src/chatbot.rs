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

    pub fn add_adapter<T>(&mut self, adapter: T)
        where T: ChatAdapter + 'static
    {
        println!("Adding adapter {}", adapter.get_name());
        // Temporarily limit the number of concurrent adapters to 1 until
        // switching to mio or similar
        assert_eq!(self.adapters.len(), 0);
        self.adapters.push(Box::new(adapter))
    }

    pub fn add_handler<T>(&mut self, handler: T)
        where T: MessageHandler + 'static
    {
        println!("Adding handler {}", handler.name());
        self.handlers.push(Box::new(handler))
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
                if handler.can_handle(msg.get_contents()) {
                    match handler.handle(&msg) {
                        Err(e) => {
                            println!("Error in handler `{}`", handler.name());
                            println!("{:?}", e);
                            println!("The incoming message was {}", msg.get_contents());

                            // TODO remove handler?
                        },
                        _ => ()
                    }
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

    #[test]
    fn test_create_chatbot() {
        let bot = Chatbot::new();
        assert_eq!(bot.get_name(), "computer");
    }

    #[test]
    fn test_chatbot_add_adapter() {
        let mut bot = Chatbot::new();
        let cli = CliAdapter::new();
        bot.add_adapter(cli);
    }

    #[test]
    fn test_chatbot_add_handler() {
        let mut bot = Chatbot::new();
        let echo = handler!("EchoHandler", r"echo .+", |_, msg| {
            Some(msg.to_owned())
        });
        bot.add_handler(echo);
    }
}
