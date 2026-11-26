//! Background discovery for the coding agents shown in Settings.

mod versions;

use std::ffi::OsString;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use coderune_agents::{detect_cli_agents_with_path, CliAgentId, CliAgentStatus};
use rune::Tx;

use self::versions::{VersionChecks, VersionInfo};

#[derive(Default)]
pub(crate) struct AgentDetection {
    pub(crate) results: Vec<CliAgentStatus>,
    pub(crate) path: Option<OsString>,
    pub(crate) check: Check,
    versions: VersionChecks,
}

#[derive(Default)]
pub(crate) enum Check {
    #[default]
    NotChecked,
    Checking(Receiver<(Vec<CliAgentStatus>, Option<OsString>)>),
    Finished,
    Failed,
}

impl AgentDetection {
    pub(crate) fn refresh(&mut self) {
        if matches!(self.check, Check::Checking(_)) {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("coderune-agent-detection".into())
            .spawn(move || {
                let _ = sender.send(detect_cli_agents_with_path());
            });
        self.check = match worker {
            Ok(_) => Check::Checking(receiver),
            Err(_) => Check::Failed,
        };
    }

    pub(crate) fn poll(&mut self, tx: &mut Tx) -> bool {
        let changed = self.poll_discovery(tx);
        self.versions.poll(tx) || changed
    }

    pub(crate) fn version(&self, agent: CliAgentId) -> Option<&VersionInfo> {
        self.versions.get(agent)
    }

    pub(crate) fn checking_versions(&self) -> bool {
        self.versions.pending()
    }

    fn poll_discovery(&mut self, tx: &mut Tx) -> bool {
        let Check::Checking(receiver) = &self.check else {
            return false;
        };
        match receiver.try_recv() {
            Ok((results, path)) => {
                self.results = results;
                self.path = path;
                self.check = Check::Finished;
                self.versions = VersionChecks::start(&self.results, self.path.as_deref());
                true
            }
            Err(TryRecvError::Disconnected) => {
                self.check = Check::Failed;
                true
            }
            Err(TryRecvError::Empty) => {
                tx.wake_after(Duration::from_millis(100));
                false
            }
        }
    }
}
