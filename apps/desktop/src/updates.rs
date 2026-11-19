//! Desktop release checks and verified package downloads.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

use directories::UserDirs;
use rune::{TextEdit, Tx};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

const RELEASES_API: &str = "https://api.github.com/repos/universitysjp/coderune/releases";
const MAX_METADATA_BYTES: u64 = 256 * 1024;
const MAX_PACKAGE_BYTES: u64 = 500 * 1024 * 1024;

type UpdateResult<T> = Result<T, &'static str>;

pub(crate) struct Updates {
    pub(crate) token: TextEdit,
    pub(crate) status: Status,
    pending: Option<Receiver<UpdateResult<Status>>>,
}

pub(crate) enum Status {
    Idle,
    Checking,
    Current,
    Available(Update),
    Downloading,
    Downloaded(PathBuf),
    Failed(&'static str),
}

impl Updates {
    pub(crate) fn new() -> Self {
        Self {
            token: TextEdit::default().secure(),
            status: Status::Idle,
            pending: None,
        }
    }

    pub(crate) fn check(&mut self) {
        self.start(Status::Checking, |token| {
            let package =
                platform_package().ok_or("No Coderune package is built for this system.")?;
            let installed =
                Version::parse(crate::VERSION).map_err(|_| "Invalid installed version.")?;
            let mut response = request(
                &format!("{RELEASES_API}/latest"),
                "application/vnd.github+json",
                token,
                Duration::from_secs(20),
            )?;
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
            self.start(Status::Downloading, move |token| {
                download(&update, token).map(Status::Downloaded)
            });
        }
    }

    fn start(
        &mut self,
        activity: Status,
        operation: impl FnOnce(&str) -> UpdateResult<Status> + Send + 'static,
    ) {
        if self.pending.is_some() {
            return;
        }
        let token = Zeroizing::new(self.token.text.trim().to_owned());
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("coderune-updates".into())
            .spawn(move || {
                let _ = sender.send(operation(&token));
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
    asset_id: u64,
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
    id: u64,
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
        .find(|asset| {
            asset.name == package && asset.id > 0 && (1..=MAX_PACKAGE_BYTES).contains(&asset.size)
        })
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
        asset_id: asset.id,
        size: asset.size,
        digest: checksum,
    }))
}

fn request(
    url: &str,
    accept: &str,
    token: &str,
    timeout: Duration,
) -> UpdateResult<ureq::http::Response<ureq::Body>> {
    let agent = ureq::Agent::config_builder()
        .https_only(true)
        .redirect_auth_headers(ureq::config::RedirectAuthHeaders::Never)
        .timeout_global(Some(timeout))
        .build()
        .new_agent();
    let mut request = agent.get(url).header("Accept", accept);
    if !token.is_empty() {
        request = request.header("Authorization", format!("Bearer {token}"));
    }
    request.call().map_err(|error| match error {
        ureq::Error::StatusCode(401 | 403) => {
            "GitHub rejected access. Check the token's repository access."
        }
        ureq::Error::StatusCode(404) => {
            "No release is accessible. Private releases need a GitHub token."
        }
        _ => "Could not reach the GitHub release. Try again.",
    })
}

fn download(update: &Update, token: &str) -> UpdateResult<PathBuf> {
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
        &format!("{RELEASES_API}/assets/{}", update.asset_id),
        "application/octet-stream",
        token,
        Duration::from_secs(180),
    )?;
    write_verified(response.body_mut().as_reader(), &mut file, update)?;
    file.sync_all()
        .map_err(|_| "Could not save the update package.")?;
    drop(file);
    fs::rename(partial, folder.path().join(update.package))
        .map_err(|_| "Could not finish saving the update package.")?;
    Ok(folder.keep())
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
                {"id": 40, "name": "Coderune-macos-arm64.zip", "size": 3, "digest": null},
                {"id": 41, "name": PACKAGE, "size": 3, "digest": ABC_DIGEST}
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
            (update.version, update.package, update.asset_id),
            (Version::new(0, 10, 0), PACKAGE, 41)
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
