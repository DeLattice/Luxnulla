use serde::{Deserialize, Serialize};

pub const CONFIG_DIR: &str = "luxnulla";

pub const LUXNULLA_CONFIG_FILE: &str = "luxnulla.kdl";

pub const XRAY_CONFIG_FILE: &str = "xray.json";
pub const XRAY_CHECKER_CONFIG_FILE: &str = "xray_checker.json";

pub const SOCKET_NAME: &str = "luxnulla-core.sock";
pub const EDITOR_NAME: &str = "zeditor";

#[derive(Deserialize, Serialize)]
pub enum CommandRequest {
    EditXray,
    EditLuxnulla,
    Status,
    Restart,
}

#[derive(Deserialize, Serialize)]
pub enum CommandResponse {
    Ok(OkCommandResponse),
    Err(ErrorCommandResponse),
}

#[derive(Deserialize, Serialize)]
pub enum OkCommandResponse {
    Message(String),
    GetSubs(Vec<String>),
}

#[derive(Deserialize, Serialize)]
pub enum ErrorCommandResponse {
    Message(String),
    GetSubs(String),
}
