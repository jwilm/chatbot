computer
========

**NOT READY TO USE**

Tea, Earl Grey, Hot.

Your very own LCARS in the form of a chat bot. There's nothing workable yet, but
the plan is to provide adapters for slack and IRC out of the box.

## Usage

The construction is inspired by Hubot's extensibility. It does require that you
write a touch of code to kick things off since Rust is a compiled language and
everything that goes along with that.

```rust

use std::env;

use computer::Chatbot;
use computer::adapter::SlackAdapter;
use computer::adapter::IrcAdapter;

use custom::handlers;

fn main() {
    // Create an instance of the bot
    let mut bot = Chatbot::new();

    // Add some connections. I guess this serves IRC and Slack
    let irc_adapter = Irc::from_config("/path/to/irc/config").unwrap();
    let slack_adapter = Slack::from_config("/path/to/slack/config").unwrap();

    bot.add_adapter(irc_adapter);
    bot.add_adapter(slack_adapter);

    // Include our custom handlers. I think when I'm done writing this, I'll
    // figure out how to hook it up to a coffee delivery service.
    bot.add_handler(custom::handlers::BlueBottleOrder::new());
    bot.add_handler(custom::handlers::BadPuns::new());

    // Run the bot. This blocks until the bot is asked to shutdown.
    bot.run();
}
```
