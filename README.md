chatbot
=======

An extensible chatbot written in rust.

[![Build Status](https://travis-ci.org/jwilm/chatbot.svg?branch=master)](https://travis-ci.org/jwilm/chatbot)

## About

The construction is inspired by Hubot's extensibility. There is an ever-growing
list of [service adapters][] and [message handlers][] as part of the project.

To get started, you might make a `main` function that looks like the following.
Once you get that running, check out the [documentation][] to add more packaged
[message handlers][] or write your own.

```rust
#[macro_use(handler)]
extern crate chatbot;

use chatbot::Chatbot;
use chatbot::adapter::CliAdapter;

fn main() {
    let mut bot = Chatbot::new("chatbot_name");

    let echo = handler!("EchoHandler", r"echo .+", |_, msg| {
        Some(msg.to_owned())
    });

    bot.add_handler(echo);
    bot.add_adapter(CliAdapter::new());

    bot.run();
}
```

## Plans

Check out the [issue tracker][] for an up-to-date list of plans for the chat
bot.

## Contributing

Contributions are very welcome on this project. To get started, fork the repo
and clone it locally. You should be able to just do `cargo run` and get a
working ping and github handler on the command line. If you want to run the test
program using the Slack adapter, do `cargo run -- --adapter slack`.

[service adapters]: https://docs.rs/chatbot/*/chatbot/adapter/trait.ChatAdapter.html#implementors
[message handlers]: https://docs.rs/chatbot/*/chatbot/handler/trait.MessageHandler.html#implementors
[documentation]: https://docs.rs/chatbot
[issue tracker]: https://github.com/jwilm/chatbot/issues
