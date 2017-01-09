#[macro_use(handler)]
extern crate chatbot;

use chatbot::Chatbot;
use chatbot::adapter::CliAdapter;

fn main() {
    let mut bot = Chatbot::new("echobot");

    let echo = handler!("EchoHandler", r"echo .+", |_, msg| {
        Some(msg.to_owned())
    });

    bot.add_handler(echo);
    bot.add_adapter(CliAdapter::new("bot"));

    bot.run();
}
