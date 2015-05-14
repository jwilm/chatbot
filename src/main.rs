extern crate computer;

use computer::Chatbot;
use computer::adapter::CliAdapter;
use computer::handler::EchoHandler;

fn main() {
    let mut bot = Chatbot::new();
    let cli_adapter = Box::new(CliAdapter::new());
    let echo_handler = Box::new(EchoHandler::new());

    bot.add_adapter(cli_adapter);
    bot.add_handler(echo_handler);

    bot.run();
}
