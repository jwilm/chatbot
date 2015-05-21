extern crate chatbot;

use chatbot::Chatbot;
use chatbot::adapter::SlackAdapter;
use chatbot::handler::EchoHandler;
use chatbot::handler::GithubIssueLinker;

#[allow(dead_code)]
fn main() {
    let mut bot = Chatbot::new();

    bot.add_adapter(Box::new(SlackAdapter::new()));
    bot.add_handler(Box::new(EchoHandler::new()));
    bot.add_handler(Box::new(GithubIssueLinker::new()));

    bot.run();
}
