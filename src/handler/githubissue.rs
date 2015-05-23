extern crate regex;

use std::io::Read;

use hyper::Client;
use hyper::header::UserAgent;
use hyper::status::StatusCode;
use regex::Captures;
use regex::Regex;
use rustc_serialize::json::Json;

use handler::MessageHandler;
use handler::HandlerResult;
use message::IncomingMessage;

/// Respond to github issue links with the title and hyperlink
pub struct GithubIssueLinker;

impl GithubIssueLinker {
    pub fn new() -> GithubIssueLinker {
        GithubIssueLinker
    }

    pub fn can_handle(&self, msg: &str) -> bool {
        let re = regex!(r"https://github.com/(?P<owner>\w|\w\w|\w[\w-]+\w)/(?P<repo>[\w_-]+)/issues/(?P<issue>\d+)");
        re.is_match(msg)
    }

    pub fn get_captures<'a>(&self, msg: &'a str) -> Option<Captures<'a>> {
        let re = regex!(r"https://github.com/(?P<owner>\w|\w\w|\w[\w-]+\w)/(?P<repo>[\w_-]+)/issues/(?P<issue>\d+)");
        re.captures(msg)
    }
}

impl MessageHandler for GithubIssueLinker {
    fn name(&self) -> &str {
        "GithubIssueLinker"
    }

    fn handle(&self, incoming: &IncomingMessage) -> HandlerResult {
        if !self.can_handle(incoming.get_contents()) {
            return Ok(())
        }

        let captures = self.get_captures(incoming.get_contents()).unwrap();
        let owner = captures.name("owner").unwrap();
        let repo = captures.name("repo").unwrap();
        let issue = captures.name("issue").unwrap();

        let mut client = Client::new();
        let url = format!("https://api.github.com/repos/{}/{}/issues/{}", owner, repo, issue);
        let mut res = client.get(&url)
            .header(UserAgent("chatbot.rs".to_owned()))
            .send().unwrap();

        // Read the Response.
        if res.status == StatusCode::Ok {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();

            let data = Json::from_str(body.as_ref()).unwrap();
            let obj = data.as_object().unwrap();

            let html_url = obj.get("html_url").unwrap().as_string().unwrap();
            let title = obj.get("title").unwrap().as_string().unwrap();

            let outgoing = format!("{} ({})", title, html_url);
            incoming.reply(outgoing).unwrap();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use handler::GithubIssueLinker;
    use message::AdapterMsg;
    use message::IncomingMessage;
    use handler::MessageHandler;
    use std::sync::mpsc::channel;

    #[test]
    #[allow(unused_variables)]
    fn test_create() {
        let handler = GithubIssueLinker::new();
    }

    #[test]
    fn test_valid_urls() {
        let handler = GithubIssueLinker::new();

        assert!(handler.can_handle("https://github.com/jwilm/chatbot/issues/123"));
        assert!(handler.can_handle("https://github.com/NaMespacE/ano_ther-repo/issues/123"));
        assert!(!handler.can_handle("https://github.com/-hyphen/word/issues/123"));
        assert!(!handler.can_handle("https://github.com/hyphen-/word/issues/123"));
        assert!(!handler.can_handle("https://github.com/hyphens--/word/issues/123"));
    }

    #[test]
    fn test_response() {
        let handler = GithubIssueLinker::new();
        let msg = "words and words https://github.com/rust-lang/rust/issues/1";
        let (tx, rx) = channel();
        let inc = IncomingMessage::new(handler.name().to_owned(),
            None, None, None, msg.to_owned(), tx);

        handler.handle(&inc).unwrap();

        match rx.recv().unwrap() {
            AdapterMsg::Outgoing(_) => {
                assert!(true);
            },
            _ => panic!("Did not receive message from handler")
        };
    }
}
