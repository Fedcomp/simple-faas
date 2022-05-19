use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
