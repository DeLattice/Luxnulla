use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Trojan {
    user_id: String,
    password: String,
    address: String,
    port: u16,
    sni: Option<String>,
    ws_path: Option<String>,
    host: Option<String>,
    allow_insecure: bool,
    name: Option<String>,
    extras: HashMap<String, String>,
}
