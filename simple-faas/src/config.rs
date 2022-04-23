use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub functions: HashMap<String, FunctionData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionData {
    pub image: String,
}
