use adapter::ChatAdapter;

pub struct CliAdapter {
    name: &'static str
}

impl CliAdapter {
    pub fn new() -> CliAdapter {
        CliAdapter {
            name: "cli"
        }
    }
}

impl ChatAdapter for CliAdapter {
    fn get_name(&self) -> &str {
        self.name
    }
}

