use crate::services::{ConfigService, StatusService};
use luxnulla::{CommandRequest, CommandResponse, OkCommandResponse};
use std::path::PathBuf;

pub struct CommandHandler {
    status_service: StatusService,
    config_service: ConfigService,
}

impl CommandHandler {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            status_service: StatusService::new(),
            config_service: ConfigService::new(config_dir),
        }
    }

    pub async fn handle_command(&self, request: CommandRequest) -> CommandResponse {
        match request {
            CommandRequest::Status => self.status_service.get_status(),

            CommandRequest::Restart => {
                CommandResponse::Ok(OkCommandResponse::Message(String::from(
                    "xray is restarted TODO",
                )))
                // self.subscription_service
                // .refresh_default_subscription()
                // .await
            }

            CommandRequest::EditXray => self.config_service.edit_xray_config().await,

            CommandRequest::EditLuxnulla => self.config_service.edit_luxnulla_config().await,
        }
    }
}
