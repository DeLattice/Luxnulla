use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Shadowsocks {
    method: String,
    password: String,
    address: String,
    port: u16,
    name: Option<String>,
    extras: HashMap<String, String>,
}
