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
    child: Mutex<Option<Child>>,
    sender: broadcast::Sender<String>,
    sender_handle: Mutex<Option<task::JoinHandle<()>>>,
    config_file_path: PathBuf,
    log_file_path: PathBuf,
}

impl XrayService {
    pub fn new(config_file_path: PathBuf, log_file_path: PathBuf) -> XrayService {
        let (sender, _) = broadcast::channel(128);
        XrayService {
            child: Mutex::new(None),
            sender,
            sender_handle: Mutex::new(None),
            config_file_path,
            log_file_path,
        }
    }

    pub fn logs(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }

    pub async fn status(&self) -> bool {
        match self.child.lock().await.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(maybe_code) => maybe_code.is_none(),
                Err(_) => false,
            },
            None => false,
        }
    }

    pub async fn start(&self) -> bool {
        let mut child = self.child.lock().await;
        let mut sender_handle = self.sender_handle.lock().await;

        if child.is_some() {
            return false;
        }

        let stdout_file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
        {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Не удалось открыть файл логов: {}", err);
                return false;
            }
        };

        let stderr_file = stdout_file.try_clone().unwrap();

        let Ok(process) = Command::new("xray")
            .args(["run", "-c", self.config_file_path.to_str().unwrap()])
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
        else {
            return false;
        };
        *child = Some(process);

        let sender = self.sender.clone();
        let log_path = self.log_file_path.clone();

        *sender_handle = Some(tokio::spawn(async move {
            // Ждем появления файла
            let file = loop {
                if let Ok(f) = File::open(&log_path).await {
                    break f;
                }
                sleep(Duration::from_millis(250)).await;
            };

            let mut reader = BufReader::new(file);

            let _ = reader.seek(SeekFrom::End(0)).await;

            let mut line_buf = String::new();

            loop {
                line_buf.clear();

                match reader.read_line(&mut line_buf).await {
                    Ok(0) => {
                        sleep(Duration::from_millis(100)).await;
                    }
                    Ok(_) => {
                        let clean_line = line_buf.trim_end();
                        if !clean_line.is_empty() {
                            let _ = sender.send(clean_line.to_string());
                        }
                    }
                    Err(e) => {
                        eprintln!("Ошибка чтения: {}", e);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }));

        true
    }

    pub async fn stop(&self) -> bool {
        let mut child = self.child.lock().await;
        let mut sender_handle = self.sender_handle.lock().await;

        if child.is_none()
            && sender_handle
                .as_ref()
                .map(|v| v.is_finished())
                .unwrap_or(true)
        {
            return false;
        }

        if let Some(mut child) = child.take() {
            let _ = child.kill().await;
        }
        if let Some(handle) = sender_handle.take() {
            handle.abort();
        }

        true
    }
}
