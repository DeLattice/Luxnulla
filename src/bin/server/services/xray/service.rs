use std::{io::SeekFrom, path::PathBuf, process::Stdio, time::Duration};

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncSeekExt, BufReader},
    process::{Child, Command},
    sync::{Mutex, broadcast},
    task,
    time::sleep,
};

pub struct XrayService {
    started: Mutex<bool>,
    sender: broadcast::Sender<String>,
    sender_handle: Mutex<Option<task::JoinHandle<()>>>,
    child: Option<Child>,
    config_file_path: PathBuf,
    log_file_path: PathBuf,
}

impl XrayService {
    pub fn new(config_file_path: PathBuf, log_file_path: PathBuf) -> XrayService {
        let (sender, _) = broadcast::channel(128);
        XrayService {
            started: Mutex::new(false),
            sender,
            sender_handle: Mutex::new(None),
            child: None,
            config_file_path,
            log_file_path,
        }
    }

    pub fn logs(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    pub async fn status(&self) -> bool {
        *self.started.lock().await
    }

    pub async fn start(&self) -> bool {
        let mut started = self.started.lock().await;
        let mut sender_handle = self.sender_handle.lock().await;

        if *started {
            return false;
        }

        let log_file_std = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path);

        if let Err(e) = log_file_std {
            eprintln!("Не удалось открыть файл логов: {}", e);
            return false;
        }
        let log_file_std = log_file_std.unwrap();

        let log_file_err = log_file_std.try_clone().unwrap();

        let cmd = Command::new("xray")
            // .args(["run", "-c", self.config_file_path.to_str().unwrap()])
            .stdout(Stdio::from(log_file_std))
            .stderr(Stdio::from(log_file_err))
            .spawn();

        if cmd.is_err() {
            return false;
        }

        self.child = Some(cmd);

        let sender = self.sender.clone();
        let log_path = self.log_file_path.clone();

        *started = true;

        *sender_handle = Some(tokio::spawn(async move {
            let mut file = loop {
                if let Ok(f) = File::open(&log_path).await {
                    break f;
                }
                sleep(Duration::from_millis(500)).await;
            };

            let _ = file.seek(SeekFrom::End(0)).await;

            let mut reader = BufReader::new(file);
            let mut line = String::new();

            loop {
                line.clear();

                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        sleep(Duration::from_millis(200)).await;
                    }
                    Ok(_) => {
                        let _ = sender.send(line.clone());
                    }
                    Err(e) => {
                        eprintln!("Ошибка чтения лога: {}", e);
                        break;
                    }
                }
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
