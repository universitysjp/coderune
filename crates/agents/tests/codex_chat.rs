#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

use coderune_agents::chat::{
    load_models, ChatEvent, ChatOptions, ChatProvider, ChatRun, ModelCatalogRun,
};

struct Project(PathBuf);

impl Project {
    fn with_server(script: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::temp_dir().join(format!(
            "coderune-codex-chat-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&base).unwrap();
        let project = Self(base);
        let executable = project.0.join("codex");
        let handshake = r#"#!/bin/sh
set -eu
test "$1" = app-server
IFS= read -r initialize
case "$initialize" in *'"method":"initialize"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":1,"result":{}}'
IFS= read -r initialized
case "$initialized" in *'"method":"initialized"'*) ;; *) exit 1 ;; esac
"#;
        fs::write(&executable, format!("{handshake}{script}")).unwrap();
        fs::set_permissions(executable, fs::Permissions::from_mode(0o700)).unwrap();
        project
    }

    fn start(&self, session: Option<&str>) -> ChatRun {
        ChatRun::start(
            ChatProvider::Codex,
            &self.0.join("codex"),
            &self.0,
            None,
            session,
            &ChatOptions {
                model: Some("test-model".into()),
                effort: Some("high".into()),
            },
            "Line one\nLine two",
        )
        .unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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
            Err(error) => panic!("Codex did not emit an event: {error}"),
        }
    }
}

#[test]
fn discovers_models_and_their_efforts_across_pages() {
    let project = Project::with_server(
        r#"
IFS= read -r first
case "$first" in *'"method":"model/list"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":2,"result":{"data":[{"model":"test-model","displayName":"Test Model","isDefault":true,"supportedReasoningEfforts":[{"reasoningEffort":"high"}]}],"nextCursor":"page-two"}}'
IFS= read -r second
case "$second" in *'"cursor":"page-two"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":3,"result":{"data":[{"model":"other-model","displayName":"Other Model"}],"nextCursor":null}}'
"#,
    );
    let models = load_models(
        ChatProvider::Codex,
        &project.0.join("codex"),
        &project.0,
        None,
        None,
    )
    .unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].label, "Test Model");
    assert!(models[0].is_default);
    assert_eq!(models[0].efforts[0].id, "high");
    assert_eq!(models[1].id, "other-model");
}

#[test]
fn abandoning_a_model_lookup_closes_the_agent_connection() {
    let project = Project::with_server(
        r#"
IFS= read -r request
case "$request" in *'"method":"model/list"'*) ;; *) exit 1 ;; esac
touch lookup-started
if IFS= read -r unexpected; then exit 1; fi
touch lookup-stopped
"#,
    );
    let lookup = ModelCatalogRun::start(
        ChatProvider::Codex,
        &project.0.join("codex"),
        &project.0,
        None,
        None,
    )
    .unwrap();
    let wait_for = |marker| {
        let deadline = Instant::now() + Duration::from_secs(3);
        let file = project.0.join(marker);
        while !file.exists() {
            assert!(Instant::now() < deadline, "agent did not write {marker}");
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    wait_for("lookup-started");
    assert!(matches!(lookup.try_recv(), Err(TryRecvError::Empty)));
    drop(lookup);
    wait_for("lookup-stopped");
}

#[test]
fn resumes_a_chat_with_the_selected_options_and_streams_one_reply() {
    let project = Project::with_server(
        r#"
IFS= read -r resume
case "$resume" in *'"method":"thread/resume"'*'"threadId":"existing"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":2,"result":{"thread":{"id":"existing"}}}'
IFS= read -r send
case "$send" in *'"effort":"high"'*'"text":"Line one\nLine two"'*'"model":"test-model"'*) ;; *) exit 1 ;; esac
printf '%s\n' \
  '{"id":3,"result":{"turn":{"id":"turn"}}}' \
  '{"method":"item/agentMessage/delta","params":{"threadId":"existing","turnId":"turn","itemId":"message","delta":"Hello"}}' \
  '{"method":"item/agentMessage/delta","params":{"threadId":"unrelated","turnId":"turn","itemId":"hidden","delta":"Other thread"}}' \
  '{"method":"item/agentMessage/delta","params":{"threadId":"existing","turnId":"turn","itemId":"message","delta":" again"}}' \
  '{"method":"item/completed","params":{"threadId":"existing","turnId":"turn","item":{"id":"message","type":"agentMessage","text":"Hello again"}}}' \
  '{"method":"turn/completed","params":{"threadId":"existing","turn":{"id":"turn","status":"completed"}}}'
"#,
    );
    let run = project.start(Some("existing"));
    assert!(matches!(next(&run), ChatEvent::Session(id) if id == "existing"));
    let mut reply = String::new();
    loop {
        match next(&run) {
            ChatEvent::Delta(delta) => reply.push_str(&delta),
            ChatEvent::Finished(result) => {
                result.unwrap();
                break;
            }
            event => panic!("Unexpected event: {event:?}"),
        }
    }
    assert_eq!(reply, "Hello again");
}

#[test]
fn sends_an_explicit_one_time_approval_for_the_displayed_command() {
    let project = Project::with_server(
        r#"
IFS= read -r start
case "$start" in *'"method":"thread/start"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"id":2,"result":{"thread":{"id":"fresh"}}}'
IFS= read -r send
printf '%s\n' \
  '{"id":3,"result":{"turn":{"id":"turn"}}}' \
  '{"id":"approval-1","method":"item/commandExecution/requestApproval","params":{"threadId":"fresh","turnId":"turn","command":"cat README.md","cwd":"/project"}}'
IFS= read -r answer
case "$answer" in *'"id":"approval-1"'*'"decision":"accept"'*) ;; *) exit 1 ;; esac
printf '%s\n' '{"method":"turn/completed","params":{"threadId":"fresh","turn":{"id":"turn","status":"completed"}}}'
"#,
    );
    let run = project.start(None);
    assert!(run.answer_approval(true).is_err());
    assert!(matches!(next(&run), ChatEvent::Session(id) if id == "fresh"));
    assert!(
        matches!(next(&run), ChatEvent::Approval { summary } if summary.contains("cat README.md") && summary.contains("/project"))
    );
    run.answer_approval(true).unwrap();
    assert!(matches!(next(&run), ChatEvent::Finished(Ok(()))));
}

#[test]
fn cancels_an_active_turn_using_the_provider_protocol() {
    let project = Project::with_server(
        r#"
IFS= read -r start
printf '%s\n' '{"id":2,"result":{"thread":{"id":"fresh"}}}'
IFS= read -r send
printf '%s\n' '{"id":3,"result":{"turn":{"id":"turn"}}}'
IFS= read -r interrupt
case "$interrupt" in *'"method":"turn/interrupt"'*'"threadId":"fresh"'*'"turnId":"turn"'*) ;; *) exit 1 ;; esac
printf '%s\n' interrupted > interrupted.txt
printf '%s\n' '{"id":4,"result":{}}'
"#,
    );
    let run = project.start(None);
    assert!(matches!(next(&run), ChatEvent::Session(_)));
    run.cancel();
    assert!(matches!(next(&run), ChatEvent::Finished(Err(reason)) if reason == "Turn cancelled."));
    assert_eq!(
        fs::read_to_string(project.0.join("interrupted.txt")).unwrap(),
        "interrupted\n"
    );
}
