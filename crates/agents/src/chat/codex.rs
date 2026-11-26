//! Codex App Server's newline-delimited JSON-RPC protocol.

use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};

use super::{emit, request_approval, ChatEvent, Effort, Model, SessionProcess, TurnRequest};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(750);

pub(super) fn models(command: Command, stopped: &AtomicBool) -> Result<Vec<Model>, String> {
    let mut server = AppServer::start(command, stopped)?;
    let mut models = Vec::new();
    let mut cursor = None;
    let mut seen_cursors = HashSet::new();
    loop {
        let result = server.request(
            "model/list",
            json!({ "cursor": cursor, "includeHidden": false }),
            stopped,
        )?;
        let page: ModelPage = serde_json::from_value(result)
            .map_err(|error| format!("Could not read the Codex model catalog: {error}"))?;
        models.extend(page.data.into_iter().map(Model::from));
        let Some(next) = page.next_cursor else {
            return Ok(models);
        };
        if !seen_cursors.insert(next.clone()) {
            return Err("Codex returned a repeating model catalog cursor.".into());
        }
        cursor = Some(next);
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelPage {
    data: Vec<CodexModel>,
    next_cursor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexModel {
    model: String,
    display_name: String,
    #[serde(default)]
    is_default: bool,
    #[serde(default)]
    supported_reasoning_efforts: Vec<CodexEffort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexEffort {
    reasoning_effort: String,
}

impl From<CodexModel> for Model {
    fn from(model: CodexModel) -> Self {
        Self {
            id: model.model,
            label: model.display_name,
            is_default: model.is_default,
            efforts: model
                .supported_reasoning_efforts
                .into_iter()
                .map(|effort| Effort {
                    label: effort.reasoning_effort.clone(),
                    id: effort.reasoning_effort,
                })
                .collect(),
        }
    }
}

pub(super) fn run(
    command: Command,
    request: &TurnRequest,
    events: &SyncSender<ChatEvent>,
    stopped: &AtomicBool,
    approvals: &Receiver<bool>,
) -> Result<(), String> {
    let cwd = command
        .get_current_dir()
        .and_then(std::path::Path::to_str)
        .map(str::to_owned);
    let mut server = AppServer::start(command, stopped)?;
    let thread = match &request.session {
        Some(id) => server.request(
            "thread/resume",
            json!({ "threadId": id, "cwd": cwd, "excludeTurns": true }),
            stopped,
        )?,
        None => server.request("thread/start", json!({ "cwd": cwd }), stopped)?,
    };
    let thread_id = thread["thread"]["id"]
        .as_str()
        .ok_or("Codex did not return a session ID.")?;

    let input_id = server.send_request(
        "turn/start",
        json!({
            "threadId": thread_id,
            "cwd": cwd,
            "input": [{"type": "text", "text": request.prompt, "text_elements": []}],
            "model": request.options.model,
            "effort": request.options.effort,
        }),
    )?;
    let mut turn = Turn {
        thread_id,
        turn_id: None,
        input_id,
        events,
        approvals,
        stopped,
        messages: HashMap::new(),
        file_changes: HashMap::new(),
    };
    let result = turn.stream(&mut server);
    if stopped.load(Ordering::Relaxed) {
        if let Some(turn_id) = &turn.turn_id {
            server.process.set_timeout(SHUTDOWN_TIMEOUT);
            let _ = server.request(
                "turn/interrupt",
                json!({ "threadId": thread_id, "turnId": turn_id }),
                &AtomicBool::new(false),
            );
        }
    }
    server.process.finish();
    result
}

struct Turn<'a> {
    thread_id: &'a str,
    turn_id: Option<String>,
    input_id: u64,
    events: &'a SyncSender<ChatEvent>,
    approvals: &'a Receiver<bool>,
    stopped: &'a AtomicBool,
    messages: HashMap<String, String>,
    file_changes: HashMap<String, Vec<String>>,
}

impl Turn<'_> {
    fn stream(&mut self, server: &mut AppServer) -> Result<(), String> {
        while let Some(message) = server.process.recv(self.stopped)? {
            let Some(method) = message["method"].as_str() else {
                if message["id"] == self.input_id {
                    let result = rpc_result(message)?;
                    self.mark_started(server, result["turn"]["id"].as_str())?;
                }
                continue;
            };
            let params = &message["params"];
            let current = params["threadId"] == self.thread_id
                && self
                    .turn_id
                    .as_deref()
                    .is_none_or(|turn| params["turnId"].as_str().is_none_or(|id| id == turn));
            if let Some(id) = message.get("id") {
                self.handle_request(server, method, id, params, current)?;
                continue;
            }
            if !current {
                continue;
            }
            match method {
                "turn/started" => self.mark_started(server, params["turn"]["id"].as_str())?,
                "item/agentMessage/delta" => {
                    if let (Some(id), Some(delta)) =
                        (params["itemId"].as_str(), params["delta"].as_str())
                    {
                        self.emit_text(id, delta, false)?;
                    }
                }
                "item/started" if params["item"]["type"] == "fileChange" => {
                    let item = &params["item"];
                    if let (Some(id), Some(changes)) =
                        (item["id"].as_str(), item["changes"].as_array())
                    {
                        let paths = changes
                            .iter()
                            .filter_map(|change| change["path"].as_str().map(str::to_owned))
                            .collect();
                        self.file_changes.insert(id.to_owned(), paths);
                    }
                }
                "item/completed" => {
                    let item = &params["item"];
                    if let Some(id) = item["id"].as_str() {
                        self.file_changes.remove(id);
                        if item["type"] == "agentMessage" {
                            if let Some(text) = item["text"].as_str() {
                                self.emit_text(id, text, true)?;
                            }
                        }
                    }
                }
                "turn/completed" => {
                    let turn = &params["turn"];
                    if turn["id"].as_str() == self.turn_id.as_deref() {
                        return match turn["status"].as_str() {
                            Some("completed") => Ok(()),
                            Some("interrupted") => Err("Turn cancelled.".into()),
                            _ => Err(message_text(
                                &turn["error"],
                                "Codex could not complete the turn.",
                            )),
                        };
                    }
                }
                _ => {}
            }
        }
        Err("Codex exited before completing the turn.".into())
    }

    fn mark_started(
        &mut self,
        server: &mut AppServer,
        turn_id: Option<&str>,
    ) -> Result<(), String> {
        if self.turn_id.is_none() {
            self.turn_id = Some(turn_id.ok_or("Codex did not return a turn ID.")?.to_owned());
            server.process.clear_timeout();
            emit(self.events, ChatEvent::Session(self.thread_id.into()))?;
        }
        Ok(())
    }

    fn emit_text(&mut self, id: &str, text: &str, complete: bool) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }
        let preceding_message = !self.messages.is_empty();
        let message = self.messages.entry(id.to_owned()).or_default();
        let delta = if complete {
            text.strip_prefix(message.as_str()).unwrap_or_default()
        } else {
            text
        };
        if delta.is_empty() {
            return Ok(());
        }
        if preceding_message && message.is_empty() {
            emit(self.events, ChatEvent::Delta("\n\n".into()))?;
        }
        message.push_str(delta);
        emit(self.events, ChatEvent::Delta(delta.into()))
    }

    fn handle_request(
        &self,
        server: &mut AppServer,
        method: &str,
        id: &Value,
        params: &Value,
        current: bool,
    ) -> Result<(), String> {
        let result = match method {
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
                let can_accept = params["availableDecisions"]
                    .as_array()
                    .is_none_or(|choices| choices.iter().any(|choice| choice == "accept"));
                let summary = (current && can_accept)
                    .then(|| approval_summary(method, params, &self.file_changes))
                    .flatten();
                let accepted = summary
                    .map(|summary| {
                        request_approval(self.events, self.approvals, self.stopped, summary)
                    })
                    .transpose()?
                    .unwrap_or(false);
                let decision = if accepted {
                    "accept"
                } else if params["availableDecisions"]
                    .as_array()
                    .is_some_and(|choices| !choices.iter().any(|choice| choice == "decline"))
                {
                    "cancel"
                } else {
                    "decline"
                };
                json!({ "decision": decision })
            }
            "item/permissions/requestApproval" => json!({ "permissions": {}, "scope": "turn" }),
            "mcpServer/elicitation/request" => json!({ "action": "decline", "content": null }),
            "item/tool/requestUserInput" => json!({ "answers": {} }),
            _ => return server.unsupported(id),
        };
        server
            .process
            .send_json(&json!({ "id": id, "result": result }))
    }
}

fn approval_summary(
    method: &str,
    params: &Value,
    file_changes: &HashMap<String, Vec<String>>,
) -> Option<String> {
    let mut summary = match method {
        "item/commandExecution/requestApproval" if !params["networkApprovalContext"].is_null() => {
            let target = &params["networkApprovalContext"];
            format!(
                "Codex requests network access to {} ({})",
                target["host"].as_str()?,
                target["protocol"].as_str()?,
            )
        }
        "item/commandExecution/requestApproval" => {
            if params["kind"] == "writeStdin" {
                return None;
            }
            format!(
                "Codex requests permission to run this command in {}\n\n{}",
                params["cwd"].as_str()?,
                params["command"].as_str()?,
            )
        }
        _ if !params["grantRoot"].is_null() => return None,
        _ => {
            let paths = file_changes.get(params["itemId"].as_str()?)?;
            if paths.is_empty() {
                return None;
            }
            format!(
                "Codex requests permission to edit these files\n\n{}",
                paths.join("\n")
            )
        }
    };
    if let Some(reason) = params["reason"]
        .as_str()
        .filter(|reason| !reason.is_empty())
    {
        summary.push_str("\n\n");
        summary.push_str(reason);
    }
    Some(summary)
}

struct AppServer {
    process: SessionProcess,
    next_id: u64,
}

impl AppServer {
    fn start(mut command: Command, stopped: &AtomicBool) -> Result<Self, String> {
        command.arg("app-server");
        let mut server = Self {
            process: SessionProcess::spawn(command)
                .map_err(|error| format!("Could not start Codex: {error}"))?,
            next_id: 1,
        };
        server.process.set_timeout(STARTUP_TIMEOUT);
        server.request(
            "initialize",
            json!({ "clientInfo": { "name": "coderune", "version": env!("CARGO_PKG_VERSION") } }),
            stopped,
        )?;
        server
            .process
            .send_json(&json!({ "method": "initialized" }))?;
        Ok(server)
    }

    fn send_request(&mut self, method: &str, params: Value) -> Result<u64, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.process
            .send_json(&json!({ "id": id, "method": method, "params": params }))?;
        Ok(id)
    }

    fn request(
        &mut self,
        method: &str,
        params: Value,
        stopped: &AtomicBool,
    ) -> Result<Value, String> {
        let id = self.send_request(method, params)?;
        while let Some(message) = self.process.recv(stopped)? {
            if message["id"] == id && message.get("method").is_none() {
                return rpc_result(message);
            }
            if let (Some(id), Some(_)) = (message.get("id"), message.get("method")) {
                self.unsupported(id)?;
            }
        }
        Err(format!("Codex exited before answering {method}."))
    }

    fn unsupported(&mut self, id: &Value) -> Result<(), String> {
        self.process.send_json(&json!({
            "id": id,
            "error": { "code": -32601, "message": "This request is unavailable in Coderune." },
        }))
    }
}

fn rpc_result(mut message: Value) -> Result<Value, String> {
    if let Some(error) = message.get("error") {
        return Err(message_text(error, "Codex rejected the request."));
    }
    message
        .as_object_mut()
        .and_then(|object| object.remove("result"))
        .ok_or_else(|| "Codex returned an invalid response.".into())
}

fn message_text(error: &Value, fallback: &str) -> String {
    error["message"].as_str().unwrap_or(fallback).to_owned()
}
