//! Agent Client Protocol adapter for OpenCode and Antigravity CLI.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use serde_json::{json, Value};

use super::{emit, request_approval, ChatEvent, Effort, Model, SessionProcess, TurnRequest};

const SETUP_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) fn run(
    command: Command,
    request: &TurnRequest,
    events: &SyncSender<ChatEvent>,
    stopped: &AtomicBool,
    approvals: &Receiver<bool>,
) -> Result<(), String> {
    let mut client = Client::connect(command, stopped)?;
    let session = client.open_session(request.session.as_deref())?;
    let id = session_id(&session, request.session.as_deref())?;
    emit(events, ChatEvent::Session(id.to_owned()))?;

    let changed;
    let settings = if let Some(model) = &request.options.model {
        changed = client.select_model(id, &session, model)?;
        &changed
    } else {
        &session
    };
    let effort_option = config_option(settings, "thought_level", "effort");
    let effort = request.options.effort.as_deref().or_else(|| {
        let option = effort_option?;
        // Some agents retain a custom effort when resuming a session. Reset it
        // only when the agent advertises an explicit default choice.
        let current = option["currentValue"].as_str()?;
        (current != "default" && choices(&option["options"]).any(|(id, _)| id == "default"))
            .then_some("default")
    });
    if let Some(effort) = effort {
        let option = effort_option
            .and_then(|option| option["id"].as_str())
            .ok_or("This model no longer exposes reasoning effort. Select the default effort.")?;
        client.set_config(id, option, effort)?;
    }

    client.process.clear_timeout();
    let result = client.request_with(
        "session/prompt",
        json!({"sessionId": id, "prompt": [{"type": "text", "text": request.prompt}]}),
        |process, frame| match frame["method"].as_str() {
            Some("session/update") => {
                let params = &frame["params"];
                let update = &params["update"];
                if params["sessionId"].as_str() == Some(id)
                    && params.pointer("/_meta/opencode~1child-session").is_none()
                    && update.pointer("/_meta/opencode~1child-session").is_none()
                    && update["sessionUpdate"] == "agent_message_chunk"
                    && update["content"]["type"] == "text"
                {
                    if let Some(text) = update["content"]["text"].as_str() {
                        emit(events, ChatEvent::Delta(text.to_owned()))?;
                    }
                }
                Ok(())
            }
            Some("session/request_permission")
                if frame["params"]["sessionId"].as_str() == Some(id) =>
            {
                permission_reply(process, frame, events, stopped, approvals)
            }
            _ => reject_unhandled(process, frame),
        },
    );
    if stopped.load(Ordering::Relaxed) {
        let _ = client.process.send_json(&json!({
            "jsonrpc": "2.0", "method": "session/cancel", "params": {"sessionId": id}
        }));
    }
    client.process.finish();
    match result?["stopReason"].as_str() {
        Some("end_turn") => Ok(()),
        Some("cancelled") => Err("The agent cancelled its reply.".to_owned()),
        Some("refusal") => Err("The agent declined the request.".to_owned()),
        Some("max_tokens" | "max_turn_requests") => {
            Err("The agent reached its response limit.".to_owned())
        }
        _ => Err("The agent returned an unknown reply status.".to_owned()),
    }
}

pub(super) fn models(
    command: Command,
    selected_model: Option<&str>,
    stopped: &AtomicBool,
) -> Result<Vec<Model>, String> {
    let cleanup = AtomicBool::new(false);
    let mut client = Client::connect(command, stopped)?;
    let session = client.open_session(None)?;
    let id = session_id(&session, None)?;
    let result = model_catalog(&mut client, &session, id, selected_model);
    // Catalog probes create a private empty session. Remove it when the agent supports deletion.
    if client.supports("delete") {
        client.stopped = &cleanup;
        client.process.set_timeout(Duration::from_secs(2));
        let _ = client.request("session/delete", json!({"sessionId": id}));
    }
    client.process.finish();
    result
}

fn model_catalog(
    client: &mut Client<'_>,
    session: &Value,
    id: &str,
    selected_model: Option<&str>,
) -> Result<Vec<Model>, String> {
    let mut catalog = if let Some(option) = config_option(session, "model", "model") {
        choices(&option["options"])
            .map(|(id, label)| Model {
                id: id.to_owned(),
                label: label.to_owned(),
                is_default: option["currentValue"] == id,
                efforts: Vec::new(),
            })
            .collect::<Vec<_>>()
    } else {
        // Older ACP agents expose basic model state outside configOptions.
        let models = &session["models"];
        models["availableModels"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|model| {
                let id = model["modelId"].as_str()?;
                Some(Model {
                    id: id.to_owned(),
                    label: model["name"].as_str().unwrap_or(id).to_owned(),
                    is_default: models["currentModelId"] == id,
                    efforts: Vec::new(),
                })
            })
            .collect()
    };

    let probe = match selected_model {
        Some(selected) => catalog.iter_mut().find(|model| model.id == selected),
        None => catalog.iter_mut().find(|model| model.is_default),
    };
    if let Some(model) = probe {
        let changed;
        let settings = if model.is_default {
            session
        } else {
            changed = client.select_model(id, session, &model.id)?;
            &changed
        };
        if let Some(option) = config_option(settings, "thought_level", "effort") {
            model.efforts = choices(&option["options"])
                .filter(|(id, _)| *id != "default")
                .map(|(id, label)| Effort {
                    id: id.to_owned(),
                    label: label.to_owned(),
                })
                .collect();
        }
    }
    Ok(catalog)
}

struct Client<'a> {
    process: SessionProcess,
    cwd: PathBuf,
    capabilities: Value,
    next_id: u64,
    stopped: &'a AtomicBool,
}

impl<'a> Client<'a> {
    fn connect(mut command: Command, stopped: &'a AtomicBool) -> Result<Self, String> {
        let cwd = command
            .get_current_dir()
            .ok_or("A project directory is required to connect to the agent.")?
            .to_path_buf();
        command.arg("acp");
        let mut process = SessionProcess::spawn(command).map_err(|error| error.to_string())?;
        process.set_timeout(SETUP_TIMEOUT);
        let mut client = Self {
            process,
            cwd,
            capabilities: Value::Null,
            next_id: 1,
            stopped,
        };
        let initialized = client.request(
            "initialize",
            json!({"protocolVersion": 1, "clientCapabilities": {}, "clientInfo": {"name": "coderune", "title": "Coderune", "version": env!("CARGO_PKG_VERSION")}}),
        )?;
        if initialized["protocolVersion"].as_u64() != Some(1) {
            return Err("This agent uses an unsupported Agent Client Protocol version.".to_owned());
        }
        client.capabilities = initialized["agentCapabilities"].clone();
        Ok(client)
    }

    fn supports(&self, capability: &str) -> bool {
        self.capabilities["sessionCapabilities"][capability].is_object()
    }

    fn open_session(&mut self, existing: Option<&str>) -> Result<Value, String> {
        let mut params = json!({"cwd": self.cwd, "mcpServers": []});
        let Some(id) = existing else {
            return self.request("session/new", params);
        };
        params["sessionId"] = json!(id);
        if self.supports("resume") {
            self.request("session/resume", params)
        } else if self.capabilities["loadSession"] == true {
            self.request("session/load", params)
        } else {
            Err("This version of the agent cannot reopen saved sessions.".to_owned())
        }
    }

    fn select_model(&mut self, id: &str, settings: &Value, model: &str) -> Result<Value, String> {
        if let Some(option) = config_option(settings, "model", "model") {
            let config = option["id"].as_str().unwrap_or("model");
            return self.set_config(id, config, model);
        }
        self.request(
            "session/set_model",
            json!({"sessionId": id, "modelId": model}),
        )
    }

    fn set_config(&mut self, id: &str, config: &str, value: &str) -> Result<Value, String> {
        self.request(
            "session/set_config_option",
            json!({
                "sessionId": id, "configId": config, "value": value
            }),
        )
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.request_with(method, params, reject_unhandled)
    }

    fn request_with(
        &mut self,
        method: &str,
        params: Value,
        mut handle_message: impl FnMut(&mut SessionProcess, &Value) -> Result<(), String>,
    ) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        self.process
            .send_json(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))?;
        while let Some(mut frame) = self.process.recv(self.stopped)? {
            if frame.get("method").is_some() || frame["id"].as_u64() != Some(id) {
                handle_message(&mut self.process, &frame)?;
                continue;
            }
            if let Some(error) = frame.get("error") {
                let message = error["message"]
                    .as_str()
                    .unwrap_or("The agent rejected the request.");
                return Err(format!("{message} ({method})"));
            }
            if let Some(result) = frame.get_mut("result") {
                return Ok(result.take());
            }
            return Err(format!("The agent sent an invalid response for {method}."));
        }
        Err(format!("The agent stopped before responding to {method}."))
    }
}

fn session_id<'a>(value: &'a Value, existing: Option<&'a str>) -> Result<&'a str, String> {
    value["sessionId"]
        .as_str()
        .or(existing)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| "The agent did not identify the session.".to_owned())
}

fn config_option<'a>(value: &'a Value, category: &str, id: &str) -> Option<&'a Value> {
    value["configOptions"]
        .as_array()?
        .iter()
        .find(|option| option["category"] == category || option["id"] == id)
}

fn choices(options: &Value) -> impl Iterator<Item = (&str, &str)> {
    options
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|option| {
            option["options"]
                .as_array()
                .map_or_else(|| std::slice::from_ref(option), Vec::as_slice)
        })
        .filter_map(|option| {
            let value = option["value"].as_str()?;
            Some((value, option["name"].as_str().unwrap_or(value)))
        })
}

fn permission_reply(
    process: &mut SessionProcess,
    frame: &Value,
    events: &SyncSender<ChatEvent>,
    stopped: &AtomicBool,
    approvals: &Receiver<bool>,
) -> Result<(), String> {
    let params = &frame["params"];
    let option = |kind: &str| {
        params["options"].as_array()?.iter().find_map(|option| {
            (option["kind"] == kind)
                .then_some(&option["optionId"])?
                .as_str()
        })
    };
    let tool = &params["toolCall"];
    let title = tool["title"].as_str().unwrap_or("Agent tool request");
    let summary = tool.get("rawInput").map_or_else(
        || title.to_owned(),
        |input| {
            format!(
                "{title}\n{}",
                input
                    .as_str()
                    .map_or_else(|| input.to_string(), str::to_owned)
            )
        },
    );
    let decision = match option("allow_once") {
        Some(allow) if request_approval(events, approvals, stopped, summary)? => Some(allow),
        _ => option("reject_once"),
    };
    let outcome = decision.map_or_else(
        || json!({"outcome": "cancelled"}),
        |option_id| json!({"outcome": "selected", "optionId": option_id}),
    );
    process.send_json(&json!({"jsonrpc": "2.0", "id": frame["id"], "result": {"outcome": outcome}}))
}

fn reject_unhandled(process: &mut SessionProcess, frame: &Value) -> Result<(), String> {
    if let Some(id) = frame.get("id").filter(|_| frame.get("method").is_some()) {
        process.send_json(&json!({"jsonrpc": "2.0", "id": id, "error": {
            "code": -32601, "message": "Unsupported client operation"
        }}))?;
    }
    Ok(())
}
