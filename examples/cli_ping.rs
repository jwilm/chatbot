#[macro_use(handler)]
extern crate chatbot;

use chatbot::Chatbot;
use chatbot::adapter::CliAdapter;

fn main() {
    let mut bot = Chatbot::new("pingbot");

    let ping = handler!("PingHandler", r"ping", |_, _| Some("pong".to_owned()));

    bot.add_addressed_handler(ping);
    bot.add_adapter(CliAdapter::new("bot"));

    bot.run();
}
