use std::{path::PathBuf, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{Mutex, broadcast},
    task,
};
use tokio_stream::{StreamExt, wrappers::LinesStream};

pub struct XrayService {
    started: Mutex<bool>,
    sender: broadcast::Sender<String>,
    sender_handle: Mutex<Option<task::JoinHandle<()>>>,
    config_file_path: PathBuf,
}

impl XrayService {
    pub fn new(config_file_path: PathBuf) -> XrayService {
        let (sender, _) = broadcast::channel(128);
        XrayService {
            started: Mutex::new(false),
            sender,
            sender_handle: Mutex::new(None),
            config_file_path,
        }
    }

    pub fn logs(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    pub async fn start(&self) -> bool {
        let mut started = self.started.lock().await;
        let mut sender_handle = self.sender_handle.lock().await;

        if *started
            && sender_handle
                .as_ref()
                .map(|v| !v.is_finished())
                .unwrap_or(false)
        {
            return false;
        }

        let Ok(mut cmd) = Command::new("xray")
            .args(["run", "-c", self.config_file_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        else {
            return false;
        };

        let stdout = BufReader::new(cmd.stdout.take().unwrap()).lines();
        let stderr = BufReader::new(cmd.stderr.take().unwrap()).lines();
        let stdout = LinesStream::new(stdout);
        let stderr = LinesStream::new(stderr);

        let mut merged = stdout.merge(stderr);
        let sender = self.sender.clone();

        *started = true;
        *sender_handle = Some(tokio::spawn(async move {
            while let Some(Ok(out)) = merged.next().await {
                let _ = sender.send(out);
            }
        }));

        true
    }

    pub async fn stop(&self) -> bool {
        let mut started = self.started.lock().await;
        let mut sender_handle = self.sender_handle.lock().await;

        if !*started
            && sender_handle
                .as_ref()
                .map(|v| v.is_finished())
                .unwrap_or(true)
        {
            return false;
        }

        *started = false;
        if let Some(handle) = sender_handle.take() {
            handle.abort();
        }

        true
    }
}
