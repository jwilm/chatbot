extern crate startuppong;
extern crate regex;

use std::io::Write;
use std::str;

use regex::Regex;

pub use startuppong::account_from_env;

use handler::MessageHandler;
use handler::HandlerResult;
use handler::HandlerError;
use message::IncomingMessage;

pub type Account = startuppong::Account;

/// Startuppong.com get_players handler
///
/// Messages requesting to "show ping pong" or "list ping pong" will result in a dump
/// of the ladder, 1 line per player, to the requester. The response will be limited to
/// `max_entries` lines.
pub struct PrintLadder {
    account: Account,
    regex: Regex,
    max_entries: usize
}

impl PrintLadder {
    /// Create the PrintLadder handler.
    ///
    /// Requires a startuppong::Account. You will need to register on startuppong.com if you wish to
    /// use this handler.
    pub fn new(account: Account, max_entries: usize) -> PrintLadder {
        PrintLadder {
            account: account,
            regex: regex!(r"(print|list|show).+ping ?pong"),
            max_entries: max_entries
        }
    }
}

impl MessageHandler for PrintLadder {
    fn name(&self) -> &str {
        "PrintLadder"
    }

    fn re(&self) -> &Regex {
        &self.regex
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        let res = try!(startuppong::get_players(&self.account));
        let players = res.players();
        let mut reply = Vec::new();

        let len = if players.len() < self.max_entries {
            players.len()
        } else {
            self.max_entries
        };

        for i in 0..len {
            try!(write!(&mut reply, "{} - {}\n", players[i].rank, players[i].name));
        }

        Ok(try!(incoming.reply(str::from_utf8(&reply).unwrap().to_string())))
    }
}

impl From<startuppong::error::ApiError> for HandlerError {
    fn from(err: startuppong::error::ApiError) -> HandlerError {
        HandlerError::Other(Box::new(err))
    }
}

/// Add startuppong.com matches from chat
///
/// Say "add a match where Player A beat Player B" and a match will be recorded on startuppong.com.
/// The handler should reply with the old and new ranks for each player.
pub struct AddMatch {
    account: Account,
    regex: Regex,
}

impl AddMatch {
    pub fn new(account: Account) -> AddMatch {
        AddMatch {
            account: account,
            regex: regex!(r"add( a)? match where (?P<winner>[\w ]+) beat (?P<loser>[\w ]+)")
        }
    }
}

impl MessageHandler for AddMatch {
    fn name(&self) -> &str {
        "AddMatch"
    }

    fn re(&self) -> &Regex {
        &self.regex
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        let captures = self.get_captures(incoming.get_contents()).unwrap();
        let winner = captures.name("winner").unwrap().trim_matches(' ');
        let loser = captures.name("loser").unwrap().trim_matches(' ');

        try!(startuppong::add_match_with_names(&self.account, winner, loser));
        Ok(try!(incoming.reply("done!".to_string())))
    }
}

#[cfg(test)]
mod tests {
    use startuppong::Account;

    use handler::MessageHandler;
    use super::PrintLadder;
    use super::AddMatch;

    #[test]
    fn print_ladder_valid_messages() {
        let acc = Account::new("a".to_string(), "b".to_string());
        let handler = PrintLadder::new(acc, 10);

        assert!(handler.can_handle("print the pingpong ladder"));
        assert!(handler.can_handle("list the pingpong ladder"));
        assert!(handler.can_handle("show the pingpong ladder"));
        assert!(handler.can_handle("print the ping pong ladder"));
        assert!(handler.can_handle("list the ping pong ladder"));
        assert!(handler.can_handle("show the ping pong ladder"));
    }

    #[test]
    fn print_ladder_invalid_messages() {
        let acc = Account::new("a".to_string(), "b".to_string());
        let handler = PrintLadder::new(acc, 10);

        assert!(!handler.can_handle("who's on top of ping pong ladder?"));
    }

    #[test]
    fn add_match_valid_messages() {
        let acc = Account::new("a".to_string(), "b".to_string());
        let handler = AddMatch::new(acc);

        assert!(handler.can_handle("add a match where joe wilm beat collin green"));
        assert!(handler.can_handle("add a match where joe beat collin"));
        assert!(handler.can_handle("add match where joe wilm beat collin green"));
        assert!(handler.can_handle("add match where joe    beat collin   "));
    }

    #[test]
    fn add_match_invalid_messages() {
        let acc = Account::new("a".to_string(), "b".to_string());
        let handler = AddMatch::new(acc);

        assert!(!handler.can_handle("subtract a match where joe wilm beat collin green"));
        assert!(!handler.can_handle("add a where joe beat collin"));
        assert!(!handler.can_handle("add a match where beat collin"));
        // TODO validation about spaces?
    }
}
