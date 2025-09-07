use anyhow::{Context, Result};
use dirs::config_dir;
use luxnulla::{CONFIG_DIR, XRAY_CONFIG_FILE};
use std::sync::Mutex;
use tokio::process::{Child, Command};

static XRAY_CHILD: Mutex<Option<Child>> = Mutex::new(None);

pub async fn start_xray() -> Result<()> {
    spawn_xray().await.context("Failed to spawn Xray process")
}

pub fn get_xray_status() -> bool {
    let mut child_guard = XRAY_CHILD.lock().unwrap();
    if let Some(child) = child_guard.as_mut() {
        if let Ok(Some(status)) = child.try_wait() {
            *child_guard = None;
            println!("Terminated with status: {}", status);
            false
        } else {
            println!("Running");
            true
        }
    } else {
        println!("Not running");
        false
    }
}

async fn spawn_xray() -> Result<()> {
    let config_path = config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join(CONFIG_DIR)
        .join(XRAY_CONFIG_FILE);

    let child = Command::new("xray")
        .args(["run", "-c", config_path.to_str().context("Invalid path")?])
        .spawn()
        .context("Failed to spawn Xray command")?;

    *XRAY_CHILD.lock().unwrap() = Some(child);
    Ok(())
}

pub async fn stop_xray() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let child_to_kill = {
        let mut child_guard = XRAY_CHILD.lock().unwrap();
        child_guard.take()
    };

    if let Some(mut child) = child_to_kill {
        match child.kill().await {
            Ok(_) => {
                println!("Xray stopped successfully.");
                Ok(())
            }
            Err(e) => {
                println!("Failed to stop Xray: {}", e);
                Err(Box::new(e))
            }
        }
    } else {
        println!("Xray is not running.");
        Ok(())
    }
}

pub async fn restart_xray() -> Result<()> {
    stop_xray().await;
    start_xray()
        .await
        .context("Failed to start Xray after a restart attempt")?;
    println!("Xray restarted successfully.");
    Ok(())
}
