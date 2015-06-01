extern crate startuppong;
extern crate regex;

use std::io::Write;
use std::str;

use regex::Regex;
use startuppong::Account;

use handler::MessageHandler;
use handler::HandlerResult;
use handler::HandlerError;
use message::IncomingMessage;

/// Startuppong.com get_players handler
///
/// Messages requesting to "show ping pong" or "list ping pong" will result in a dump
/// of the ladder, 1 line per player, to the requester.
pub struct PrintLadder<'a> {
    account: &'a Account,
    regex: Regex
}

impl<'a> PrintLadder<'a> {
    /// Create the PrintLadder handler.
    ///
    /// Requires a reference to a startuppong::Account. You will need to register on
    /// startuppong.com if you wish to use this handler.
    pub fn new(account: &'a Account) -> PrintLadder<'a> {
        PrintLadder {
            account: account,
            regex: regex!(r"(print|list|show).+ping ?pong")
        }
    }
}

impl<'a> MessageHandler for PrintLadder<'a> {
    fn name(&self) -> &str {
        "PrintLadder"
    }

    fn re(&self) -> &Regex {
        &self.regex
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        let res = try!(startuppong::get_players(self.account));
        let players = res.players();
        let mut reply = Vec::new();

        for player in &players {
            try!(write!(&mut reply, "{} - {}\n", player.rank, player.name));
        }

        Ok(try!(incoming.reply(str::from_utf8(&reply).unwrap().to_string())))
    }
}

impl From<startuppong::error::ApiError> for HandlerError {
    fn from(err: startuppong::error::ApiError) -> HandlerError {
        HandlerError::Other(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use startuppong::Account;

    use handler::MessageHandler;
    use super::PrintLadder;

    #[test]
    fn check_valid_messages() {
        let acc = Account::new("a".to_string(), "b".to_string());
        let handler = PrintLadder::new(&acc);

        assert!(handler.can_handle("print the pingpong ladder"));
        assert!(handler.can_handle("list the pingpong ladder"));
        assert!(handler.can_handle("show the pingpong ladder"));
        assert!(handler.can_handle("print the ping pong ladder"));
        assert!(handler.can_handle("list the ping pong ladder"));
        assert!(handler.can_handle("show the ping pong ladder"));
    }

    #[test]
    fn check_invalid_messages() {
        let acc = Account::new("a".to_string(), "b".to_string());
        let handler = PrintLadder::new(&acc);

        assert!(!handler.can_handle("who's on top of ping pong ladder?"));
    }
}
