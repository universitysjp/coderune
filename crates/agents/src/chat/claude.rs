//! Claude Code's bidirectional stream-json transport.

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use serde_json::{json, Value};

use super::{emit, request_approval, ChatEvent, Effort, Model, SessionProcess, TurnRequest};

const INIT_ID: &str = "coderune-initialize";
const INIT_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) fn run(
    mut command: Command,
    request: &TurnRequest,
    events: &SyncSender<ChatEvent>,
    stopped: &AtomicBool,
    approvals: &Receiver<bool>,
) -> Result<(), String> {
    if let Some(session) = &request.session {
        command.arg(format!("--resume={session}"));
    }
    if let Some(model) = &request.options.model {
        command.arg(format!("--model={model}"));
    }
    if let Some(effort) = &request.options.effort {
        command.arg(format!("--effort={effort}"));
    }
    let (mut process, _) = connect(command, stopped)?;
    process.clear_timeout();
    process.send_json(&json!({
        "type": "user",
        "message": {"role": "user", "content": request.prompt},
        "parent_tool_use_id": null,
        "session_id": "default"
    }))?;
    let result = read_turn(&mut process, events, stopped, approvals);
    if stopped.load(Ordering::Relaxed) {
        let _ = process.send_json(&json!({
            "type": "control_request", "request_id": "coderune-interrupt",
            "request": {"subtype": "interrupt"}
        }));
    }
    process.finish();
    result
}

fn read_turn(
    process: &mut SessionProcess,
    events: &SyncSender<ChatEvent>,
    stopped: &AtomicBool,
    approvals: &Receiver<bool>,
) -> Result<(), String> {
    let mut streamed = false;
    while let Some(frame) = process.recv(stopped)? {
        match frame["type"].as_str() {
            Some("system") if frame["subtype"] == "init" => {
                if let Some(id) = frame["session_id"].as_str().filter(|id| !id.is_empty()) {
                    emit(events, ChatEvent::Session(id.to_owned()))?;
                }
            }
            Some("stream_event") if frame["parent_tool_use_id"].is_null() => {
                let event = &frame["event"];
                let segment = match event["type"].as_str() {
                    Some("content_block_delta") if event["delta"]["type"] == "text_delta" => {
                        event["delta"]["text"].as_str()
                    }
                    Some("content_block_start") if event["content_block"]["type"] == "text" => {
                        event["content_block"]["text"].as_str()
                    }
                    _ => None,
                };
                if let Some(text) = segment.filter(|text| !text.is_empty()) {
                    streamed = true;
                    emit(events, ChatEvent::Delta(text.to_owned()))?;
                }
            }
            Some("control_request") => {
                reply_to_request(process, &frame, events, stopped, approvals)?;
            }
            Some("result") => {
                if frame["is_error"] == true || frame["subtype"] != "success" {
                    return Err(result_error(&frame));
                }
                if !streamed {
                    if let Some(text) = frame["result"].as_str().filter(|text| !text.is_empty()) {
                        emit(events, ChatEvent::Delta(text.to_owned()))?;
                    }
                }
                return Ok(());
            }
            _ => {}
        }
    }
    Err("Claude Code stopped before returning a response.".to_owned())
}

pub(super) fn models(command: Command, stopped: &AtomicBool) -> Result<Vec<Model>, String> {
    let (mut process, metadata) = connect(command, stopped)?;
    process.finish();
    let models = metadata["models"]
        .as_array()
        .ok_or("This version of Claude Code did not list its models.")?;
    Ok(models
        .iter()
        .filter_map(|entry| {
            let id = entry["value"].as_str()?;
            let efforts = entry["supportedEffortLevels"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(|level| Effort {
                    id: level.to_owned(),
                    label: effort_label(level),
                })
                .collect();
            Some(Model {
                id: id.to_owned(),
                label: entry["displayName"].as_str().unwrap_or(id).to_owned(),
                efforts,
                is_default: id == "default",
            })
        })
        .collect())
}

fn connect(mut command: Command, stopped: &AtomicBool) -> Result<(SessionProcess, Value), String> {
    command.args([
        "--print",
        "--verbose",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--include-partial-messages",
        "--permission-prompt-tool",
        "stdio",
    ]);
    // The SDK removes this variable so a GUI started from a Claude terminal can connect.
    command.env_remove("CLAUDECODE");
    let mut process = SessionProcess::spawn(command).map_err(|error| error.to_string())?;
    process.set_timeout(INIT_TIMEOUT);
    process.send_json(&json!({
        "type": "control_request",
        "request_id": INIT_ID,
        "request": {"subtype": "initialize"}
    }))?;
    while let Some(mut frame) = process.recv(stopped)? {
        if frame["type"] == "result" && frame["is_error"] == true {
            return Err(result_error(&frame));
        }
        let response = &mut frame["response"];
        if response["request_id"] != INIT_ID {
            continue;
        }
        if response["subtype"] != "success" {
            return Err(response["error"]
                .as_str()
                .unwrap_or("Claude Code could not initialize.")
                .to_owned());
        }
        let metadata = response["response"].take();
        return Ok((process, metadata));
    }
    process.wait(stopped)?;
    Err("Claude Code stopped before initialization.".to_owned())
}

fn reply_to_request(
    process: &mut SessionProcess,
    frame: &Value,
    events: &SyncSender<ChatEvent>,
    stopped: &AtomicBool,
    approvals: &Receiver<bool>,
) -> Result<(), String> {
    let request = &frame["request"];
    let id = &frame["request_id"];
    if request["subtype"] != "can_use_tool" || id.is_null() {
        return process.send_json(&json!({
            "type": "control_response",
            "response": {"subtype": "error", "request_id": id, "error": "Unsupported request"}
        }));
    }
    let tool = request["tool_name"].as_str().unwrap_or("Tool");
    let input = &request["input"];
    let decision = if tool == "AskUserQuestion" {
        json!({"behavior": "deny", "message": "Ask your question in the chat so the user can reply."})
    } else {
        let title = request["title"].as_str().unwrap_or(tool);
        let details = serde_json::to_string_pretty(input).map_err(|error| error.to_string())?;
        let allowed = request_approval(events, approvals, stopped, format!("{title}\n{details}"))?;
        if allowed {
            json!({"behavior": "allow", "updatedInput": input})
        } else {
            json!({"behavior": "deny", "message": "The user denied this tool request."})
        }
    };
    process.send_json(&json!({
        "type": "control_response",
        "response": {"subtype": "success", "request_id": id, "response": decision}
    }))
}

fn result_error(frame: &Value) -> String {
    frame["result"]
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .or_else(|| {
            frame["errors"]
                .as_array()?
                .iter()
                .find_map(|error| error.as_str().or_else(|| error["message"].as_str()))
        })
        .unwrap_or("Claude Code could not finish this reply.")
        .to_owned()
}

fn effort_label(value: &str) -> String {
    match value {
        "low" => "Low",
        "medium" => "Medium",
        "high" => "High",
        "xhigh" => "Extra high",
        "max" => "Max",
        _ => value,
    }
    .to_owned()
}
