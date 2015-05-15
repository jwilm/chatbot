extern crate chatbot;

use chatbot::Chatbot;
use chatbot::adapter::CliAdapter;
use chatbot::handler::EchoHandler;

fn main() {
    let mut bot = Chatbot::new();

    bot.add_adapter(Box::new(CliAdapter::new()));
    bot.add_handler(Box::new(EchoHandler::new()));

    bot.run();
}
