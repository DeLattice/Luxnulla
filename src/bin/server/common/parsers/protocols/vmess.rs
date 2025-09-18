use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Vmess {
    user_id: String,
    address: String,
    port: u16,
    aid: u32,
    network: String,
    type_field: Option<String>,
    host: Option<String>,
    path: Option<String>,
    name: Option<String>,
    name_client: Option<String>,
    // raw parameters store
    extras: HashMap<String, String>,
}
