extern crate chatbot;
extern crate getopts;

use std::env;

use chatbot::Chatbot;
use chatbot::adapter::CliAdapter;
use chatbot::adapter::SlackAdapter;
use chatbot::handler::PingHandler;
use chatbot::handler::GithubIssueLinker;

use getopts::Options;
use getopts::ParsingStyle;

#[allow(dead_code)]
fn main() {

    let args = env::args().collect::<Vec<String>>();
    let mut opts = Options::new();
    opts.parsing_style(ParsingStyle::StopAtFirstFree);
    opts.optopt("a", "adapter", "Chat Adapter to use", "slack|cli");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string())
    };

    let adapter_name = matches.opt_str("a").unwrap_or("cli".to_owned());

    let mut bot = Chatbot::new();

    // Add adapter based on command line argument
    match adapter_name.as_ref() {
        "slack" => bot.add_adapter(Box::new(SlackAdapter::new())),
        "cli" => bot.add_adapter(Box::new(CliAdapter::new())),
        _ => panic!("Unexpected adapter name. Use 'cli' or 'slack'.")
    };

    bot.add_handler(Box::new(PingHandler::new()));
    bot.add_handler(Box::new(GithubIssueLinker::new()));

    bot.run();
}
