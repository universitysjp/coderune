//! Replacing the running app with a verified update.
//!
//! The package is already size- and SHA-256-checked in [`crate::updates`];
//! this module unpacks it beside the installed app, checks the app's identity
//! and signature, and swaps the two in one step before relaunching. On macOS
//! the swap is `renamex_np(RENAME_SWAP)`; on Windows the signed installer runs
//! silently over the current per-user install. A package this copy cannot
//! replace (Linux's `.deb`, or a read-only bundle) is handed to the person.

use std::path::{Path, PathBuf};

type InstallResult<T> = Result<T, &'static str>;

/// What happened to a downloaded package.
pub(crate) enum Prepared {
    /// Staged and verified: [`restart_into`] installs it.
    Restart(Install),
    /// Kept for the person to install, because this copy cannot replace itself.
    Manual(PathBuf),
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::CString;
    use std::fs;
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{InstallResult, Prepared};

    const BUNDLE_ID: &str = "edu.sjp.coderune";
    /// A staged update, and after a swap the bundle it replaced, live here.
    const STAGING_PREFIX: &str = ".Coderune-update-";

    pub(crate) struct Install {
        pub(crate) version: String,
        update: PathBuf,
        installed: PathBuf,
    }

    /// This executable's `.app`, when it runs from one.
    fn installed_bundle() -> Option<PathBuf> {
        let executable = std::env::current_exe().ok()?.canonicalize().ok()?;
        let bundle = executable.parent()?.parent()?.parent()?;
        (bundle
            .extension()
            .is_some_and(|extension| extension == "app")
            && bundle.join("Contents/Info.plist").is_file())
        .then(|| bundle.to_path_buf())
    }

    /// Replacing needs the bundle and its folder writable; Gatekeeper's
    /// translocated copies and system installs are not.
    fn replaceable(bundle: &Path) -> bool {
        let writable = |path: &Path| {
            CString::new(path.as_os_str().as_bytes())
                .is_ok_and(|path| unsafe { libc::access(path.as_ptr(), libc::W_OK) == 0 })
        };
        !bundle
            .components()
            .any(|component| component.as_os_str() == "AppTranslocation")
            && bundle.parent().is_some_and(writable)
            && writable(bundle)
    }

    pub(crate) fn prepare(package: &Path, version: &str) -> InstallResult<Prepared> {
        let installed = installed_bundle()
            .ok_or("Coderune is not running from an app bundle, so it cannot replace itself.")?;
        if !replaceable(&installed) {
            return Ok(Prepared::Manual(package.to_path_buf()));
        }
        let parent = installed
            .parent()
            .ok_or("The Coderune app has no folder to update in.")?;
        let staging = parent.join(format!("{STAGING_PREFIX}{version}"));
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(|_| "Could not clear the earlier update.")?;
        }
        fs::create_dir(&staging).map_err(|_| "Could not open the update folder.")?;
        if unpack(package, &staging).is_err() {
            let _ = fs::remove_dir_all(&staging);
            return Err("Could not unpack the update.");
        }
        let update = staging.join("Coderune.app");
        if let Err(error) = verify(&update, version) {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
        Ok(Prepared::Restart(Install {
            version: version.to_string(),
            update,
            installed,
        }))
    }

    fn unpack(archive: &Path, staging: &Path) -> InstallResult<()> {
        let status = Command::new("/usr/bin/ditto")
            .args(["-x", "-k"])
            .arg(archive)
            .arg(staging)
            .stdin(Stdio::null())
            .status()
            .map_err(|_| "Could not run ditto.")?;
        status
            .success()
            .then_some(())
            .ok_or("Could not unpack the update.")
    }

    /// The unpacked app must be Coderune, at the version it claims, with an
    /// intact code signature.
    fn verify(bundle: &Path, version: &str) -> InstallResult<()> {
        let plist = bundle.join("Contents/Info.plist");
        let value = |key: &str| {
            Command::new("/usr/libexec/PlistBuddy")
                .args(["-c", &format!("Print :{key}")])
                .arg(&plist)
                .stdin(Stdio::null())
                .output()
                .ok()
                .filter(|output| output.status.success())
                .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        };
        if value("CFBundleIdentifier").as_deref() != Some(BUNDLE_ID) {
            return Err("The update is not Coderune.");
        }
        if value("CFBundleShortVersionString").as_deref() != Some(version) {
            return Err("The update is not the version it claims.");
        }
        let intact = Command::new("/usr/bin/codesign")
            .args(["--verify", "--strict"])
            .arg(bundle)
            .stdin(Stdio::null())
            .output()
            .is_ok_and(|output| output.status.success());
        intact
            .then_some(())
            .ok_or("The update's code signature is not intact.")
    }

    /// Swaps the bundles in one step, so the installed path never lacks an app.
    fn swap(install: &Install) -> InstallResult<()> {
        let from = CString::new(install.update.as_os_str().as_bytes())
            .map_err(|_| "The update path is invalid.")?;
        let to = CString::new(install.installed.as_os_str().as_bytes())
            .map_err(|_| "The app path is invalid.")?;
        unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_SWAP) }
            .eq(&0)
            .then_some(())
            .ok_or("Could not swap in the updated Coderune.app.")
    }

    pub(crate) fn restart_into(install: &Install) -> InstallResult<()> {
        swap(install)?;
        Command::new("/bin/sh")
            .args([
                "-c",
                "while kill -0 \"$CODERUNE_PID\" 2>/dev/null; do sleep 0.2; done; \
                 exec /usr/bin/open \"$CODERUNE_BUNDLE\"",
            ])
            .env("CODERUNE_PID", std::process::id().to_string())
            .env("CODERUNE_BUNDLE", &install.installed)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "Could not schedule the updated Coderune.")?;
        Ok(())
    }

    /// Removes what an earlier update left beside the bundle: the replaced
    /// bundle once the new version runs, or an unpacked update never applied.
    pub(crate) fn finish_on_launch() {
        let Some(parent) =
            installed_bundle().and_then(|bundle| bundle.parent().map(Path::to_path_buf))
        else {
            return;
        };
        let Ok(entries) = fs::read_dir(&parent) else {
            return;
        };
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with(STAGING_PREFIX))
            {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }

    pub(crate) fn version(install: &Install) -> &str {
        &install.version
    }

    #[cfg(test)]
    mod tests {
        use std::fs;
        use std::path::{Path, PathBuf};
        use std::process::Command;

        use super::*;

        fn temporary(name: &str) -> PathBuf {
            let path =
                std::env::temp_dir().join(format!("coderune-update-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            path.canonicalize().unwrap()
        }

        /// A minimal ad-hoc signed `Coderune.app` with `identifier` and `version`.
        fn bundle(root: &Path, identifier: &str, version: &str) -> PathBuf {
            let bundle = root.join("Coderune.app");
            fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
            fs::copy("/usr/bin/true", bundle.join("Contents/MacOS/coderune")).unwrap();
            fs::write(
                bundle.join("Contents/Info.plist"),
                format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>\
                     <key>CFBundleIdentifier</key><string>{identifier}</string>\
                     <key>CFBundleExecutable</key><string>coderune</string>\
                     <key>CFBundleShortVersionString</key><string>{version}</string>\
                     </dict></plist>"
                ),
            )
            .unwrap();
            assert!(Command::new("/usr/bin/codesign")
                .args(["--force", "--sign", "-", "--identifier", identifier])
                .arg(&bundle)
                .status()
                .unwrap()
                .success());
            bundle
        }

        #[test]
        fn only_coderune_at_the_claimed_version_with_an_intact_signature_passes() {
            let root = temporary("verify");
            let app = bundle(&root, BUNDLE_ID, "9.9.9");
            verify(&app, "9.9.9").unwrap();
            assert!(verify(&app, "9.9.8").is_err());
            let other = bundle(&root.join("other"), "edu.sjp.other", "9.9.9");
            assert!(verify(&other, "9.9.9").is_err());
            fs::write(app.join("Contents/MacOS/coderune"), b"tampered").unwrap();
            assert!(verify(&app, "9.9.9").is_err());
            let _ = fs::remove_dir_all(root);
        }

        #[test]
        fn the_swap_exchanges_the_installed_and_staged_bundles() {
            let root = temporary("swap");
            let installed = root.join("Coderune.app");
            fs::create_dir_all(installed.join("Contents")).unwrap();
            fs::write(installed.join("Contents/version"), "old").unwrap();
            let update = root
                .join(format!("{STAGING_PREFIX}9.9.9"))
                .join("Coderune.app");
            fs::create_dir_all(update.join("Contents")).unwrap();
            fs::write(update.join("Contents/version"), "new").unwrap();

            swap(&Install {
                version: "9.9.9".to_string(),
                update: update.clone(),
                installed: installed.clone(),
            })
            .unwrap();
            assert_eq!(
                fs::read_to_string(installed.join("Contents/version")).unwrap(),
                "new"
            );
            assert_eq!(
                fs::read_to_string(update.join("Contents/version")).unwrap(),
                "old"
            );
            let _ = fs::remove_dir_all(root);
        }

        #[test]
        fn translocated_and_system_bundles_are_not_replaceable() {
            let root = temporary("replaceable");
            let app = bundle(&root, BUNDLE_ID, "9.9.9");
            assert!(replaceable(&app));
            assert!(!replaceable(&root.join("AppTranslocation/x/Coderune.app")));
            let _ = fs::remove_dir_all(root);
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use super::{InstallResult, Prepared};

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    pub(crate) struct Install {
        pub(crate) version: String,
        script: PathBuf,
    }

    /// The installer replaces the per-user install in place. It runs after
    /// this process exits, then starts Coderune again.
    pub(crate) fn prepare(package: &Path, version: &str) -> InstallResult<Prepared> {
        let executable =
            std::env::current_exe().map_err(|_| "Could not find the running Coderune.")?;
        let folder = executable
            .parent()
            .ok_or("Could not find the Coderune folder.")?;
        let script = package.with_file_name("coderune-update.bat");
        std::fs::write(&script, restart_script(package, &executable, folder))
            .map_err(|_| "Could not prepare the update.")?;
        Ok(Prepared::Restart(Install {
            version: version.to_string(),
            script,
        }))
    }

    fn restart_script(package: &Path, executable: &Path, folder: &Path) -> String {
        format!(
            "@echo off\r\n\
             ping 127.0.0.1 -n 2 >nul\r\n\
             start \"\" /wait \"{}\" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART \
             /CLOSEAPPLICATIONS /DIR=\"{}\"\r\n\
             start \"\" \"{}\"\r\n\
             del \"%~f0\"\r\n",
            package.display(),
            folder.display(),
            executable.display(),
        )
    }

    pub(crate) fn restart_into(install: &Install) -> InstallResult<()> {
        use std::os::windows::process::CommandExt;
        Command::new("cmd")
            .arg("/C")
            .arg(&install.script)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
            .spawn()
            .map_err(|_| "Could not start the Coderune installer.")?;
        Ok(())
    }

    pub(crate) fn finish_on_launch() {}

    pub(crate) fn version(install: &Install) -> &str {
        &install.version
    }

    #[cfg(test)]
    mod tests {
        use std::path::Path;

        use super::*;

        #[test]
        fn the_restart_script_closes_the_app_installs_quietly_and_reopens_it() {
            let script = restart_script(
                Path::new(r"C:\Users\ada\Downloads\Coderune-windows-x64-setup.exe"),
                Path::new(r"C:\Users\ada\AppData\Local\Programs\Coderune\coderune.exe"),
                Path::new(r"C:\Users\ada\AppData\Local\Programs\Coderune"),
            );
            assert!(script.contains("/VERYSILENT"));
            assert!(script.contains("/CLOSEAPPLICATIONS"));
            assert!(script.contains(r#"/DIR="C:\Users\ada\AppData\Local\Programs\Coderune""#));
            assert!(script.contains(
                r#"start "" "C:\Users\ada\AppData\Local\Programs\Coderune\coderune.exe""#
            ));
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    use std::path::{Path, PathBuf};

    use super::{InstallResult, Prepared};

    pub(crate) struct Install {
        version: String,
    }

    /// A `.deb` needs the system package manager, so the package is kept for
    /// the person to install.
    pub(crate) fn prepare(package: &Path, _version: &str) -> InstallResult<Prepared> {
        Ok(Prepared::Manual(package.to_path_buf()))
    }

    pub(crate) fn restart_into(_install: &Install) -> InstallResult<()> {
        Err("This copy of Coderune is installed by a package manager.")
    }

    pub(crate) fn finish_on_launch() {}

    pub(crate) fn version(install: &Install) -> &str {
        &install.version
    }
}

/// Stages a downloaded package, or says where it was kept.
pub(crate) use platform::Install;

pub(crate) fn prepare(package: &Path, version: &str) -> InstallResult<Prepared> {
    platform::prepare(package, version)
}

/// Installs a staged update and schedules the relaunch. The caller exits.
pub(crate) fn restart_into(install: &Install) -> InstallResult<()> {
    platform::restart_into(install)
}

/// Clears what an earlier update left behind.
pub(crate) fn finish_on_launch() {
    platform::finish_on_launch();
}

/// The version a staged update installs.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(crate) fn version(install: &Install) -> &str {
    platform::version(install)
}
