use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub version: u64,
    pub docker_host: String,
    pub functions: HashMap<String, FunctionData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionData {
    pub image: String,
}
