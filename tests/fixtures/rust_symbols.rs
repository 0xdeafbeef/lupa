pub struct Alpha {
    name: String,
}

impl Alpha {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn greet(&self) -> &str {
        &self.name
    }
}

pub struct Beta;

impl Beta {
    pub fn new() -> Self {
        Self
    }
}

fn parse_config() -> usize {
    1
}
