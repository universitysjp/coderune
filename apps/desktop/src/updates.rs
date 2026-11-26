//! Desktop release checks and verified package downloads.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use crate::update_install::{self, Prepared};

use directories::UserDirs;
use rune::Tx;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const RELEASES_API: &str = "https://api.github.com/repos/universitysjp/coderune/releases";
const RELEASES_DOWNLOAD: &str = "https://github.com/universitysjp/coderune/releases/download";
const MAX_METADATA_BYTES: u64 = 256 * 1024;
const MAX_PACKAGE_BYTES: u64 = 500 * 1024 * 1024;

type UpdateResult<T> = Result<T, &'static str>;

pub(crate) struct Updates {
    pub(crate) status: Status,
    pending: Option<Receiver<UpdateResult<Status>>>,
}

pub(crate) enum Status {
    Idle,
    Checking,
    Current,
    NoRelease,
    Available(Update),
    Downloading,
    /// Staged and verified: restarting installs it.
    Ready(update_install::Install),
    /// Kept for the person, because this copy cannot replace itself.
    Manual(PathBuf),
    Failed(&'static str),
}

impl Updates {
    pub(crate) fn new() -> Self {
        Self {
            status: Status::Idle,
            pending: None,
        }
    }

    pub(crate) fn check(&mut self) {
        // A staged update is the newest one this copy can install; replacing
        // it with a fresh answer would only mean downloading it again.
        if matches!(self.status, Status::Ready(_)) {
            return;
        }
        self.start(Status::Checking, || {
            let package =
                platform_package().ok_or("No Coderune package is built for this system.")?;
            let installed =
                Version::parse(crate::VERSION).map_err(|_| "Invalid installed version.")?;
            let mut response = match request(
                &format!("{RELEASES_API}/latest"),
                "application/vnd.github+json",
                Duration::from_secs(20),
            ) {
                Ok(response) => response,
                Err(ureq::Error::StatusCode(404)) => return Ok(Status::NoRelease),
                Err(ureq::Error::StatusCode(403 | 429)) => {
                    return Err("GitHub limited update checks. Try again later.");
                }
                Err(_) => return Err("Could not reach GitHub Releases. Try again."),
            };
            let contents = response
                .body_mut()
                .with_config()
                .limit(MAX_METADATA_BYTES)
                .reader();
            let update = select_release(contents, &installed, package)?;
            Ok(update.map_or(Status::Current, Status::Available))
        });
    }

    pub(crate) fn download(&mut self) {
        if let Status::Available(update) = &self.status {
            let update = update.clone();
            self.start(Status::Downloading, move || {
                let package = download(&update)?;
                let version = update.version.to_string();
                match crate::update_install::prepare(&package, &version)? {
                    Prepared::Restart(install) => {
                        // The staged bundle stands on its own; keep the
                        // download folder from growing a copy per update.
                        if let Some(folder) = package.parent() {
                            let _ = fs::remove_dir_all(folder);
                        }
                        Ok(Status::Ready(install))
                    }
                    Prepared::Manual(path) => Ok(Status::Manual(path)),
                }
            });
        }
    }

    /// Installs the staged update and restarts into it. Never returns on
    /// success, because the app it would return to is being replaced.
    pub(crate) fn restart(&mut self) {
        if let Status::Ready(install) = &self.status {
            if let Err(message) = crate::update_install::restart_into(install) {
                self.status = Status::Failed(message);
                return;
            }
            std::process::exit(0);
        }
    }

    fn start(
        &mut self,
        activity: Status,
        operation: impl FnOnce() -> UpdateResult<Status> + Send + 'static,
    ) {
        if self.pending.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("coderune-updates".into())
            .spawn(move || {
                let _ = sender.send(operation());
            });
        if worker.is_ok() {
            self.status = activity;
            self.pending = Some(receiver);
        } else {
            self.status = Status::Failed("Could not start the update check.");
        }
    }

    pub(crate) fn poll(&mut self, tx: &mut Tx) -> bool {
        let Some(pending) = &self.pending else {
            return false;
        };
        let result = match pending.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => {
                tx.wake_after(Duration::from_millis(100));
                return false;
            }
            Err(TryRecvError::Disconnected) => Err("The update operation ended unexpectedly."),
        };
        self.pending = None;
        self.status = result.unwrap_or_else(Status::Failed);
        true
    }
}

#[derive(Clone)]
pub(crate) struct Update {
    pub(crate) version: Version,
    package: &'static str,
    size: u64,
    digest: [u8; 32],
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    size: u64,
    digest: Option<String>,
}

fn platform_package() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("Coderune-linux-x64.deb"),
        ("macos", "aarch64") => Some("Coderune-macos-arm64.zip"),
        ("windows", "x86_64") => Some("Coderune-windows-x64-setup.exe"),
        _ => None,
    }
}

fn select_release(
    contents: impl Read,
    installed: &Version,
    package: &'static str,
) -> UpdateResult<Option<Update>> {
    let release: Release =
        serde_json::from_reader(contents).map_err(|_| "Invalid release metadata.")?;
    let version = release
        .tag_name
        .strip_prefix('v')
        .and_then(|version| Version::parse(version).ok())
        .filter(|version| version.pre.is_empty() && version.build.is_empty())
        .ok_or("Invalid published version.")?;
    if version <= *installed {
        return Ok(None);
    }

    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == package && (1..=MAX_PACKAGE_BYTES).contains(&asset.size))
        .ok_or("The latest release has no usable package for this system.")?;
    let digest = asset
        .digest
        .as_deref()
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .ok_or("The release package is missing a SHA-256 checksum.")?;
    let mut checksum = [0; 32];
    hex::decode_to_slice(digest, &mut checksum).map_err(|_| "Invalid package checksum.")?;
    Ok(Some(Update {
        version,
        package,
        size: asset.size,
        digest: checksum,
    }))
}

fn request(
    url: &str,
    accept: &str,
    timeout: Duration,
) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
    ureq::Agent::config_builder()
        .https_only(true)
        .timeout_global(Some(timeout))
        .build()
        .new_agent()
        .get(url)
        .header("Accept", accept)
        .call()
}

fn download(update: &Update) -> UpdateResult<PathBuf> {
    let users = UserDirs::new().ok_or("Could not find your Downloads folder.")?;
    let downloads = users
        .download_dir()
        .map_or_else(|| users.home_dir().join("Downloads"), PathBuf::from);
    fs::create_dir_all(&downloads).map_err(|_| "Could not open your Downloads folder.")?;
    let folder = tempfile::Builder::new()
        .prefix(&format!("Coderune-{}-", update.version))
        .tempdir_in(&downloads)
        .map_err(|_| "Could not create a folder in Downloads.")?;
    let partial = folder.path().join("download.partial");
    let mut file =
        File::create_new(&partial).map_err(|_| "Could not create the update package.")?;
    let mut response = request(
        &format!("{RELEASES_DOWNLOAD}/v{}/{}", update.version, update.package),
        "application/octet-stream",
        Duration::from_secs(180),
    )
    .map_err(|_| "Could not download the published package. Try again.")?;
    write_verified(response.body_mut().as_reader(), &mut file, update)?;
    file.sync_all()
        .map_err(|_| "Could not save the update package.")?;
    drop(file);
    fs::rename(partial, folder.path().join(update.package))
        .map_err(|_| "Could not finish saving the update package.")?;
    Ok(keep_package(folder, update.package))
}

/// `TempDir::keep` returns the folder, but the installer needs the package file.
fn keep_package(folder: tempfile::TempDir, package: &str) -> PathBuf {
    folder.keep().join(package)
}

fn write_verified(body: impl Read, output: &mut impl Write, update: &Update) -> UpdateResult<()> {
    let mut body = body.take(update.size + 1);
    let mut digest = Sha256::new();
    let mut size = 0;
    let mut buffer = [0; 16 * 1024];
    loop {
        let count = body
            .read(&mut buffer)
            .map_err(|_| "The package download was interrupted.")?;
        if count == 0 {
            break;
        }
        size += count as u64;
        let chunk = &buffer[..count];
        output
            .write_all(chunk)
            .map_err(|_| "Could not save the update package.")?;
        digest.update(chunk);
    }
    let actual: [u8; 32] = digest.finalize().into();
    if size != update.size || actual != update.digest {
        return Err("The downloaded package failed its integrity check.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const PACKAGE: &str = "Coderune-linux-x64.deb";
    const ABC_DIGEST: &str =
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn release() -> String {
        json!({
            "tag_name": "v0.10.0",
            "assets": [
                {"name": "Coderune-macos-arm64.zip", "size": 3, "digest": null},
                {"name": PACKAGE, "size": 3, "digest": ABC_DIGEST}
            ]
        })
        .to_string()
    }

    #[test]
    fn newer_published_release_selects_only_the_current_platform_package() {
        let update = select_release(release().as_bytes(), &Version::new(0, 9, 8), PACKAGE)
            .unwrap()
            .unwrap();
        assert_eq!(
            (update.version, update.package),
            (Version::new(0, 10, 0), PACKAGE)
        );
    }

    #[test]
    fn older_published_release_never_offers_a_downgrade() {
        assert!(
            select_release(release().as_bytes(), &Version::new(0, 11, 0), PACKAGE)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn package_with_missing_checksum_is_unavailable() {
        assert!(select_release(
            release().as_bytes(),
            &Version::new(0, 9, 0),
            "Coderune-macos-arm64.zip",
        )
        .is_err());
    }

    #[test]
    fn a_finished_download_points_at_the_package_file() {
        let folder = tempfile::Builder::new()
            .prefix("coderune-download-test-")
            .tempdir()
            .unwrap();
        let package = "Coderune-macos-arm64.zip";
        fs::write(folder.path().join(package), b"package").unwrap();

        let path = keep_package(folder, package);

        assert!(path.is_file());
        assert_eq!(path.file_name().unwrap(), package);
        let _ = fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn download_requires_matching_bytes_and_length() {
        let update = select_release(release().as_bytes(), &Version::new(0, 9, 0), PACKAGE)
            .unwrap()
            .unwrap();
        let mut verified = Vec::new();
        write_verified(&b"abc"[..], &mut verified, &update).unwrap();
        assert_eq!(verified, b"abc");
        for invalid in [&b"xyz"[..], &b"ab"[..], &b"abcd"[..]] {
            assert!(write_verified(invalid, &mut Vec::new(), &update).is_err());
        }
    }
}
