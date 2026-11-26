//! Bounded pipe I/O and process lifetime for coding-agent protocols.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

const POLL_INTERVAL: Duration = Duration::from_millis(50);
const MAX_LINE_BYTES: u64 = 16 * 1024 * 1024;

pub(super) struct SessionProcess {
    child: Child,
    input: Option<SyncSender<Vec<u8>>>,
    output: Receiver<io::Result<String>>,
    deadline: Option<Instant>,
}

impl SessionProcess {
    pub(super) fn spawn(mut command: Command) -> io::Result<Self> {
        // stderr can contain local paths, prompts, and credentials. Keep it
        // separate from the protocol and use structured errors for feedback.
        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let (input, lines) = mpsc::sync_channel::<Vec<u8>>(8);
        let (sender, output) = mpsc::sync_channel(16);
        let mut process = Self {
            child,
            input: Some(input),
            output,
            deadline: None,
        };
        let mut stdin = process
            .child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("Agent input pipe is unavailable."))?;
        let stdout = process
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Agent output pipe is unavailable."))?;
        thread::Builder::new()
            .name("coderune-chat-input".into())
            .spawn(move || {
                while let Ok(bytes) = lines.recv() {
                    if stdin.write_all(&bytes).is_err() {
                        break;
                    }
                }
            })?;
        thread::Builder::new()
            .name("coderune-chat-output".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match (&mut reader).take(MAX_LINE_BYTES + 1).read_line(&mut line) {
                        Ok(0) => break,
                        Ok(size) if size as u64 <= MAX_LINE_BYTES => {
                            if sender.send(Ok(line)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {
                            let _ =
                                sender.send(Err(io::Error::other("Agent event was too large.")));
                            break;
                        }
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            break;
                        }
                    }
                }
            })?;
        Ok(process)
    }

    pub(super) fn set_timeout(&mut self, timeout: Duration) {
        self.deadline = Some(Instant::now() + timeout);
    }

    pub(super) fn clear_timeout(&mut self) {
        self.deadline = None;
    }

    pub(super) fn send_json(&mut self, value: &Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        self.input
            .as_ref()
            .ok_or_else(|| "The coding agent input is closed.".to_string())?
            .try_send(bytes)
            .map_err(|error| format!("Could not send a request to the coding agent: {error}"))
    }

    fn close_input(&mut self) {
        self.input.take();
    }

    /// Give the provider time to flush its session state and exit on stdin EOF.
    pub(super) fn finish(&mut self) {
        self.close_input();
        self.set_timeout(Duration::from_millis(750));
        let _ = self.wait(&AtomicBool::new(false));
    }

    pub(super) fn recv(&mut self, stopped: &AtomicBool) -> Result<Option<Value>, String> {
        while let Some(line) = self.recv_line(stopped)? {
            if let Ok(value) = serde_json::from_str(&line) {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn recv_line(&mut self, stopped: &AtomicBool) -> Result<Option<String>, String> {
        loop {
            self.check_wait(stopped)?;
            match self.output.recv_timeout(POLL_INTERVAL) {
                Ok(line) => return line.map(Some).map_err(|error| error.to_string()),
                Err(RecvTimeoutError::Disconnected) => return Ok(None),
                Err(RecvTimeoutError::Timeout) => continue,
            }
        }
    }

    pub(super) fn wait(&mut self, stopped: &AtomicBool) -> Result<(), String> {
        loop {
            self.check_wait(stopped)?;
            if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!("Coding agent exited ({status})."))
                };
            }
            thread::sleep(POLL_INTERVAL);
        }
    }

    fn check_wait(&self, stopped: &AtomicBool) -> Result<(), String> {
        if stopped.load(Ordering::Relaxed) {
            return Err("Turn cancelled.".into());
        }
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err("The coding agent did not respond in time.".into());
        }
        Ok(())
    }
}

impl Drop for SessionProcess {
    fn drop(&mut self) {
        if self.input.is_some() {
            self.finish();
        }
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
