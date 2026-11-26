//! Open a project folder in the desktop terminal.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) fn open(folder: &Path) -> io::Result<()> {
    if !folder.is_absolute() || !folder.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The project folder is unavailable.",
        ));
    }
    if let Some(terminal) = std::env::var_os("TERMINAL").filter(|value| !value.is_empty()) {
        return launch(Command::new(terminal).current_dir(folder));
    }
    open_default(folder)
}

#[cfg(target_os = "macos")]
fn open_default(folder: &Path) -> io::Result<()> {
    let status = Command::new("/usr/bin/open")
        .args(["-b", "com.apple.Terminal"])
        .arg(folder)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("Terminal returned {status}.")))
    }
}

#[cfg(target_os = "linux")]
fn open_default(folder: &Path) -> io::Result<()> {
    let mut directory = std::ffi::OsString::from("--dir=");
    directory.push(folder);
    match launch(
        Command::new("xdg-terminal-exec")
            .arg(directory)
            .current_dir(folder),
    ) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            launch(Command::new("x-terminal-emulator").current_dir(folder))
        }
        result => result,
    }
}

#[cfg(windows)]
fn open_default(folder: &Path) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x10;
    let shell = std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into());
    // Windows opens a new console using the user's default terminal host.
    let child = Command::new(shell)
        .current_dir(folder)
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()?;
    reap(child);
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
fn open_default(_: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Set TERMINAL to your terminal executable.",
    ))
}

fn launch(command: &mut Command) -> io::Result<()> {
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    reap(child);
    Ok(())
}

fn reap(mut child: std::process::Child) {
    std::thread::spawn(move || {
        let _ = child.wait();
    });
}
