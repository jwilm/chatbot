#[macro_use(handler)]
extern crate chatbot;
extern crate getopts;

use std::env;

use chatbot::Chatbot;
use chatbot::adapter::CliAdapter;
use chatbot::adapter::SlackAdapter;
use chatbot::adapter::IrcAdapter;
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

    let mut bot = Chatbot::new("chatbotbot");

    // Add adapter based on command line argument
    match adapter_name.as_ref() {
        "slack" => bot.add_adapter(SlackAdapter::new()),
        "cli" => bot.add_adapter(CliAdapter::new()),
        "irc" => {
            let config = chatbot::adapter::irc::Config {
                nickname: Some(format!("chatbotbot")),
                alt_nicks: Some(vec![format!("chatbot_rs"), format!("chatbotrs")]),
                server: Some(format!("irc.mozilla.org")),
                channels: Some(vec![format!("#chatbot")]),
                .. Default::default()
            };
            bot.add_adapter(IrcAdapter::new(config))
        },
        _ => panic!("Unexpected adapter name. Use 'cli' or 'slack'.")
    };

    let ping = handler!("PingHandler", r"ping", |_, _| { Some("pong".to_owned()) });

    let robot_name = "Mr. T";
    let trout = handler!("TroutSlap", r"slap (?P<user>.+)", move |matches, _| {
        match matches.name("user") {
            Some(user) => {
                Some(format!("{} slaps {} around a bit with a large trout",
                             robot_name, user))
            },
            None => None
        }
    });

    let echo = handler!("EchoHandler", r"echo (?P<msg>.+)", |matches, _| {
        matches.name("msg").map(|msg| { msg.to_owned() })
    });

    bot.add_handler(ping);
    bot.add_handler(trout);
    bot.add_handler(echo);
    bot.add_handler(GithubIssueLinker::new());

    bot.run();
}
