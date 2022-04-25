#[derive(Debug, Clone)]
pub struct Client {
    host: String,
}

impl Client {
    pub fn new(host: String) -> Self {
        Client { host }
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}
