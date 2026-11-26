#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

use coderune_agents::chat::{load_models, ChatEvent, ChatOptions, ChatProvider, ChatRun};

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
}

impl Fixture {
    fn new(script: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "coderune-native-chat-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&directory).unwrap();
        let executable = directory.join("agent");
        fs::write(&executable, format!("#!/bin/sh\nset -eu\n{script}")).unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            directory,
            executable,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn next(run: &ChatRun) -> ChatEvent {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match run.try_recv() {
            Ok(event) => return event,
            Err(TryRecvError::Empty) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("The agent did not emit an event: {error}"),
        }
    }
}

#[test]
fn opencode_reports_efforts_for_the_selected_project_model() {
    let agent = Fixture::new(
        r#"
test "$1" = acp
IFS= read -r init
case "$init" in *'"method":"initialize"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1,"agentCapabilities":{"sessionCapabilities":{"delete":{}}}}}'
IFS= read -r session
case "$session" in *'"method":"session/new"'*'"cwd":"'*'"mcpServers":[]'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"catalog","configOptions":[{"id":"model","category":"model","currentValue":"provider/default","options":[{"group":"provider","name":"Provider","options":[{"value":"provider/default","name":"Default Model"},{"value":"provider/selected","name":"Selected Model"}]}]}]}}'
IFS= read -r model
case "$model" in *'"method":"session/set_config_option"'*'"configId":"model"'*'"value":"provider/selected"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"configOptions":[{"id":"effort","category":"thought_level","currentValue":"default","options":[{"value":"default","name":"Default"},{"value":"depth","name":"Deep"}]}]}}'
IFS= read -r delete
case "$delete" in *'"method":"session/delete"'*'"sessionId":"catalog"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"jsonrpc":"2.0","id":4,"result":{}}'
"#,
    );
    let models = load_models(
        ChatProvider::OpenCode,
        &agent.executable,
        &agent.directory,
        None,
        Some("provider/selected"),
    )
    .unwrap();
    assert_eq!(models.len(), 2);
    assert!(models[0].is_default);
    assert_eq!(models[1].label, "Selected Model");
    assert_eq!(models[1].efforts.len(), 1);
    assert_eq!(models[1].efforts[0].label, "Deep");
}

#[test]
fn opencode_clears_a_saved_effort_when_default_is_selected() {
    let agent = Fixture::new(
        r#"
test "$1" = acp
IFS= read -r init
case "$init" in *'"method":"initialize"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":1,"result":{"protocolVersion":1,"agentCapabilities":{"sessionCapabilities":{"resume":{}}}}}'
IFS= read -r session
case "$session" in *'"method":"session/resume"'*'"sessionId":"existing"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":2,"result":{"sessionId":"existing","configOptions":[{"id":"effort","category":"thought_level","currentValue":"high","options":[{"value":"default","name":"Default"},{"value":"high","name":"High"}]}]}}'
IFS= read -r effort
case "$effort" in *'"method":"session/set_config_option"'*'"configId":"effort"'*'"value":"default"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":3,"result":{}}'
IFS= read -r prompt
case "$prompt" in *'"method":"session/prompt"'*'"text":"What changed?"'*) ;; *) exit 1 ;; esac
printf '%s\n' \
  '{"method":"session/update","params":{"sessionId":"existing","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"Response at default effort"}}}}' \
  '{"id":4,"result":{"stopReason":"end_turn"}}'
"#,
    );
    let run = ChatRun::start(
        ChatProvider::OpenCode,
        &agent.executable,
        &agent.directory,
        None,
        Some("existing"),
        &ChatOptions::default(),
        "What changed?",
    )
    .unwrap();
    assert!(matches!(next(&run), ChatEvent::Session(id) if id == "existing"));
    assert!(matches!(next(&run), ChatEvent::Delta(text) if text == "Response at default effort"));
    assert!(matches!(next(&run), ChatEvent::Finished(Ok(()))));
}

#[test]
fn claude_resumes_the_chat_and_updates_it_after_explicit_tool_approval() {
    let agent = Fixture::new(
        r#"
case "$*" in *'--resume=existing --model=sonnet --effort=high'*'--permission-prompt-tool stdio'*) ;; *) exit 1 ;; esac
IFS= read -r init
case "$init" in *'"subtype":"initialize"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"type":"control_response","response":{"subtype":"success","request_id":"coderune-initialize","response":{}}}'
IFS= read -r prompt
case "$prompt" in *'"content":"Line one\nLine two"'*) ;; *) exit 1 ;; esac
printf '%s\n' \
  '{"type":"system","subtype":"init","session_id":"existing"}' \
  '{"type":"control_request","request_id":"approval","request":{"subtype":"can_use_tool","tool_name":"Read","input":{"file_path":"README.md"}}}'
IFS= read -r decision
case "$decision" in *'"request_id":"approval"'*'"behavior":"allow"'*'"file_path":"README.md"'*) ;; *) exit 1 ;; esac
printf '%s\n' \
  '{"type":"stream_event","parent_tool_use_id":"child","event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"Skip this child text"}}}' \
  '{"type":"stream_event","parent_tool_use_id":null,"event":{"type":"content_block_delta","delta":{"type":"text_delta","text":"Hello"}}}' \
  '{"type":"stream_event","parent_tool_use_id":null,"event":{"type":"content_block_delta","delta":{"type":"text_delta","text":" again"}}}' \
  '{"type":"result","subtype":"success","is_error":false,"result":"Hello again"}'
"#,
    );
    let run = ChatRun::start(
        ChatProvider::ClaudeCode,
        &agent.executable,
        &agent.directory,
        None,
        Some("existing"),
        &ChatOptions {
            model: Some("sonnet".into()),
            effort: Some("high".into()),
        },
        "Line one\nLine two",
    )
    .unwrap();
    assert!(matches!(next(&run), ChatEvent::Session(id) if id == "existing"));
    assert!(
        matches!(next(&run), ChatEvent::Approval { summary } if summary.contains("Read") && summary.contains("README.md"))
    );
    run.answer_approval(true).unwrap();
    assert!(matches!(next(&run), ChatEvent::Delta(text) if text == "Hello"));
    assert!(matches!(next(&run), ChatEvent::Delta(text) if text == " again"));
    assert!(matches!(next(&run), ChatEvent::Finished(Ok(()))));
}
