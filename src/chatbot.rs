use adapter::ChatAdapter;

pub struct Chatbot {
    name: String,
    adapters: Vec<Box<ChatAdapter>>
}

impl Chatbot {
    pub fn new() -> Chatbot {
        Chatbot {
            name: "computer".to_owned(),
            adapters: Vec::new()
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn add_adapter<T: ChatAdapter + 'static>(&mut self, adapter: Box<T>) {
        self.adapters.push(adapter)
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
        let cli = Box::new(CliAdapter::new());
        bot.add_adapter(cli);
    }
}
