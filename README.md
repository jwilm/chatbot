chatbot
=======

**NOT READY TO USE**

An extensible chatbot written in rust

## Usage

The construction is inspired by Hubot's extensibility. It does require that you
write your own `main()` since we cannot just load scripts dynamically.

```rust

extern crate chatbot;

use std::env;

use chatbot::Chatbot;
use chatbot::adapter::SlackAdapter; // TODO
use chatbot::adapter::IrcAdapter; // TODO

use chatbot::handlers::EchoHandler;

// Add your own handlers
use custom::handlers;

fn main() {
    // Create an instance of the bot
    let mut bot = Chatbot::new();

    // Add some connections. I guess this serves IRC and Slack
    let irc_adapter = Irc::from_config("/path/to/irc/config").unwrap();
    let slack_adapter = Slack::from_config("/path/to/slack/config").unwrap();

    bot.add_adapter(irc_adapter);
    bot.add_adapter(slack_adapter);

    // Add some built in handlers
    bot.add_handler(Box::new(EchoHandler::new()));

    // Include our custom handlers. I think when I'm done writing this, I'll
    // figure out how to hook it up to a coffee delivery service.
    bot.add_handler(custom::handlers::BlueBottleOrder::new());
    bot.add_handler(custom::handlers::BadPuns::new());

    // Run the bot. This blocks until the bot is asked to shutdown.
    bot.run();
}
```
