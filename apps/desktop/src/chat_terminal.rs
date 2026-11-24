//! Interactive coding-agent sessions. Terminal output stays in memory.

use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::thread;
use std::time::Duration;

use coderune_ui::theme::{self, CODE_SM};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use rune::{col, grid, text, Color, El, GridRow, Key, KeyEvent, Span, Tx};

const CELL_WIDTH: f32 = 6.6;
const CELL_HEIGHT: f32 = 15.0;
const INSET: f32 = 8.0;
const SCROLLBACK_ROWS: usize = 1_000;
const CHUNK: usize = 4_096;
const IO_QUEUE: usize = 32;
const MAX_LINE_BYTES: usize = 64 * 1_024;

pub(crate) struct Terminal {
    screen: vt100::Parser<TerminalReplies>,
    // Drop the output receiver before closing the PTY so the reader cannot
    // remain blocked writing to a full queue during teardown.
    output: Option<Receiver<Vec<u8>>>,
    process: Option<Process>,
    status: Option<String>,
}

struct Process {
    child: Box<dyn Child + Send + Sync>,
    input: SyncSender<Vec<u8>>,
    master: Box<dyn MasterPty + Send>,
}

impl Drop for Process {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) && self.child.kill().is_ok() {
            let _ = self.child.wait();
        }
    }
}

impl Terminal {
    /// Starts the selected executable directly in its project directory.
    pub(crate) fn launch(executable: &Path, cwd: &Path, path: Option<&OsStr>) -> io::Result<Self> {
        if !executable.is_absolute() || !cwd.is_absolute() || !cwd.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Choose an installed agent and an available project folder.",
            ));
        }
        let size = size(24, 80);
        let pair = native_pty_system()
            .openpty(size)
            .map_err(io::Error::other)?;
        let mut reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        let mut writer = pair.master.take_writer().map_err(io::Error::other)?;
        let mut command = agent_command(executable)?;
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env_remove("NO_COLOR");
        if let Some(path) = path {
            command.env("PATH", path);
        }
        let child = pair
            .slave
            .spawn_command(command)
            .map_err(io::Error::other)?;
        drop(pair.slave);

        let (input, input_rx) = mpsc::sync_channel::<Vec<u8>>(IO_QUEUE);
        let process = Process {
            child,
            master: pair.master,
            input,
        };
        let (output_tx, output) = mpsc::sync_channel(IO_QUEUE);
        thread::Builder::new()
            .name("coderune-terminal-output".into())
            .spawn(move || loop {
                let mut bytes = vec![0; CHUNK];
                match reader.read(&mut bytes) {
                    Ok(0) => break,
                    Ok(count) => {
                        bytes.truncate(count);
                        if output_tx.send(bytes).is_err() {
                            break;
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            })?;
        thread::Builder::new()
            .name("coderune-terminal-input".into())
            .spawn(move || {
                while let Ok(bytes) = input_rx.recv() {
                    if writer.write_all(&bytes).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            screen: vt100::Parser::new_with_callbacks(
                size.rows,
                size.cols,
                SCROLLBACK_ROWS,
                TerminalReplies::default(),
            ),
            process: Some(process),
            output: Some(output),
            status: None,
        })
    }

    pub(crate) fn is_running(&self) -> bool {
        self.process.is_some()
    }

    pub(crate) fn stop(&mut self) -> io::Result<()> {
        if let Some(process) = &mut self.process {
            if process.child.try_wait()?.is_none() {
                process.child.kill()?;
            }
            self.status = Some("Agent stopped".into());
        }
        Ok(())
    }

    /// Sends one composer line to a running interactive agent.
    pub(crate) fn send_line(&mut self, text: &str) -> io::Result<()> {
        if text.len() > MAX_LINE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The message exceeds the terminal's 64 KiB input limit.",
            ));
        }
        let text: String = text.chars().filter(|ch| !ch.is_control()).collect();
        if text.trim().is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Write a message first.",
            ));
        }
        let mut line = Vec::with_capacity(text.len() + 13);
        if self.screen.screen().bracketed_paste() {
            line.extend_from_slice(b"\x1b[200~");
            line.extend_from_slice(text.as_bytes());
            line.extend_from_slice(b"\x1b[201~");
        } else {
            line.extend_from_slice(text.as_bytes());
        }
        line.push(b'\r');
        self.send(line)
    }

    pub(crate) fn poll(&mut self, tx: &mut Tx) -> bool {
        let changed = self.read_output();
        if self.is_running() || self.output.is_some() {
            tx.wake_after(Duration::from_millis(32));
        }
        changed
    }

    fn read_output(&mut self) -> bool {
        let mut changed = false;
        if let Some(output) = &self.output {
            for _ in 0..IO_QUEUE {
                match output.try_recv() {
                    Ok(bytes) => {
                        self.screen.process(&bytes);
                        changed = true;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.output = None;
                        break;
                    }
                }
            }
        }
        if let Some(process) = &mut self.process {
            let reply = std::mem::take(&mut self.screen.callbacks_mut().bytes);
            if !reply.is_empty() {
                if let Err(mpsc::TrySendError::Full(reply)) = process.input.try_send(reply) {
                    self.screen.callbacks_mut().bytes = reply;
                }
            }
            match process.child.try_wait() {
                Ok(None) => {}
                Ok(Some(exit)) => {
                    self.status
                        .get_or_insert_with(|| format!("Agent exited ({exit})"));
                    self.finish_process();
                    changed = true;
                }
                Err(error) => {
                    self.status = Some(format!("Could not check agent process: {error}"));
                    self.finish_process();
                    changed = true;
                }
            }
        }
        changed
    }

    fn finish_process(&mut self) {
        let process = self.process.take();
        // Closing ConPTY may block while the reader flushes the last output.
        // Keep polling its bounded queue from the UI while the handles close.
        #[cfg(windows)]
        let _ = thread::Builder::new()
            .name("coderune-terminal-close".into())
            .spawn(move || drop(process));
        #[cfg(not(windows))]
        drop(process);
    }

    fn send(&mut self, bytes: Vec<u8>) -> io::Result<()> {
        let process = self.process.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Start the coding agent before sending.",
            )
        })?;
        process.input.try_send(bytes).map_err(|error| match error {
            mpsc::TrySendError::Full(_) => io::Error::new(
                io::ErrorKind::WouldBlock,
                "The terminal input queue is full.",
            ),
            mpsc::TrySendError::Disconnected(_) => io::Error::new(
                io::ErrorKind::BrokenPipe,
                "The agent stopped accepting input.",
            ),
        })?;
        self.screen.screen_mut().set_scrollback(0);
        Ok(())
    }

    fn resize(&mut self, width: f32, height: f32, cell_width: f32) {
        if width < cell_width || height < CELL_HEIGHT {
            return;
        }
        let dimension = |points: f32, cell: f32| (points / cell).floor().clamp(1.0, 240.0) as u16;
        let cols = dimension(width, cell_width);
        let rows = dimension(height, CELL_HEIGHT);
        if self.screen.screen().size() == (rows, cols) {
            return;
        }
        if let Some(process) = &self.process {
            let mut pty_size = size(rows, cols);
            pty_size.pixel_width = (f32::from(cols) * cell_width).round() as u16;
            if let Err(error) = process.master.resize(pty_size) {
                self.status = Some(format!("Could not resize terminal: {error}"));
                return;
            }
        }
        self.screen.screen_mut().set_size(rows, cols);
    }

    fn scroll(&mut self, lines: isize) -> bool {
        let screen = self.screen.screen_mut();
        let before = screen.scrollback();
        screen.set_scrollback(before.saturating_add_signed(lines));
        screen.scrollback() != before
    }

    pub(crate) fn view<S: 'static>(&self, current: fn(&mut S) -> Option<&mut Self>) -> El {
        let screen = self.screen.screen();
        let (rows, cols) = screen.size();
        let cell_width = rune::text::advance("M", CODE_SM).max(1.0);
        let cursor = (self.is_running() && !screen.hide_cursor() && screen.scrollback() == 0)
            .then(|| screen.cursor_position());
        let lines = (0..rows)
            .map(|row| Rc::new(grid_row(screen, row, cols, cursor)))
            .collect();
        let mut surface = grid(cols, cell_width, CELL_HEIGHT, lines)
            .key("chat-terminal")
            .label("Coding agent terminal. Press F6 for panel controls.")
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .focusable(true)
            .on_layout(move |state: &mut S, [_, _, width, height]| {
                if let Some(terminal) = current(state) {
                    terminal.resize(width, height, cell_width);
                }
            })
            .on_wheel(move |state: &mut S, wheel, _| {
                let lines = (wheel.dy / CELL_HEIGHT).round() as isize;
                current(state).is_some_and(|terminal| terminal.scroll(lines))
            })
            .on_key(move |state: &mut S, key, tx| {
                if key.key == Key::Function(6) {
                    tx.focus("dock-left", true);
                    return true;
                }
                let Some(terminal) = current(state) else {
                    return false;
                };
                if key.mods.shift {
                    let page = terminal.screen.screen().size().0 as isize;
                    match key.key {
                        Key::PageUp => return terminal.scroll(page),
                        Key::PageDown => return terminal.scroll(-page),
                        _ => {}
                    }
                }
                let application_cursor = terminal.screen.screen().application_cursor();
                let Some(bytes) = terminal_input(key, tx.key_base(), application_cursor) else {
                    return false;
                };
                if let Err(error) = terminal.send(bytes) {
                    if !terminal.is_running() {
                        return false;
                    }
                    terminal.status = Some(error.to_string());
                }
                true
            });
        if let Some((row, col)) = cursor {
            surface = surface.grid_caret(col, row);
        }
        let mut pane = col()
            .w_full()
            .grow(1.0)
            .min_h(0.0)
            .padding(INSET)
            .fill(theme::canvas())
            .clip_rounded(0.0)
            .child(surface);
        if let Some(status) = &self.status {
            pane = pane.child(
                text(status)
                    .font(theme::BODY_SM)
                    .color(theme::muted())
                    .wrap(),
            );
        }
        pane
    }
}

#[derive(Default)]
struct TerminalReplies {
    bytes: Vec<u8>,
}

impl vt100::Callbacks for TerminalReplies {
    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        first: Option<u8>,
        second: Option<u8>,
        params: &[&[u16]],
        command: char,
    ) {
        // A CLI can query the terminal in the middle of a redraw and wait
        // synchronously for a reply before showing its prompt.
        if second.is_some() || self.bytes.len() >= CHUNK {
            return;
        }
        let number = params
            .first()
            .and_then(|param| param.first())
            .copied()
            .unwrap_or(0);
        match (first, command, number) {
            (None, 'c', 0) => self.bytes.extend_from_slice(b"\x1b[?1;2c"),
            (Some(b'>'), 'c', 0) => self.bytes.extend_from_slice(b"\x1b[>0;0;0c"),
            (None, 'n', 5) => self.bytes.extend_from_slice(b"\x1b[0n"),
            (None, 'n', 6) | (Some(b'?'), 'n', 6) => {
                let (row, col) = screen.cursor_position();
                let prefix = if first.is_some() { "?" } else { "" };
                self.bytes
                    .extend_from_slice(format!("\x1b[{prefix}{};{}R", row + 1, col + 1).as_bytes());
            }
            _ => {}
        }
    }
}

#[cfg(not(windows))]
fn agent_command(executable: &Path) -> io::Result<CommandBuilder> {
    Ok(CommandBuilder::new(executable))
}

#[cfg(windows)]
fn agent_command(executable: &Path) -> io::Result<CommandBuilder> {
    let batch = executable
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cmd") || ext.eq_ignore_ascii_case("bat"));
    if !batch {
        return Ok(CommandBuilder::new(executable));
    }

    let root = std::env::var_os("SystemRoot").map(std::path::PathBuf::from);
    let shell = root.map(|root| root.join("System32").join("cmd.exe"));
    let shell = shell
        .filter(|path| path.is_absolute() && path.is_file())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Windows Command Prompt could not be located.",
            )
        })?;
    let mut quoted_path = std::ffi::OsString::from("\"");
    quoted_path.push(executable.as_os_str());
    quoted_path.push("\"");
    // ConPTY uses CreateProcessW, which expects an executable. Pass the entire
    // quoted launcher path as one environment value to a fixed cmd expression.
    let mut command = CommandBuilder::new(shell);
    command.args(["/d", "/v:off", "/c", "%CODERUNE_AGENT_LAUNCHER%"]);
    command.env("CODERUNE_AGENT_LAUNCHER", quoted_path);
    Ok(command)
}

fn size(rows: u16, cols: u16) -> PtySize {
    PtySize {
        rows,
        cols,
        pixel_width: (f32::from(cols) * CELL_WIDTH).round() as u16,
        pixel_height: (f32::from(rows) * CELL_HEIGHT).round() as u16,
    }
}

fn terminal_input(key: &KeyEvent, base: Option<char>, application_cursor: bool) -> Option<Vec<u8>> {
    if key.mods.logo || (key.mods.cmd && !key.mods.ctrl) {
        return None;
    }
    if key.mods.ctrl && !key.mods.alt {
        let control = match key.key {
            Key::Command(ch) => Some(ch),
            _ => base,
        };
        match control?.to_ascii_uppercase() {
            ch @ '@'..='_' => return Some(vec![ch as u8 & 0x1f]),
            ' ' => return Some(vec![0]),
            _ => return None,
        }
    }

    let cursor_key = |normal: &'static [u8], application: &'static [u8]| {
        if application_cursor {
            application
        } else {
            normal
        }
    };
    let bytes: &[u8] = match &key.key {
        Key::Text(text) => text.as_bytes(),
        Key::Enter => b"\r",
        Key::Backspace => b"\x7f",
        Key::Escape => b"\x1b",
        Key::Tab if key.mods.shift => b"\x1b[Z",
        Key::Tab => b"\t",
        Key::Up => cursor_key(b"\x1b[A", b"\x1bOA"),
        Key::Down => cursor_key(b"\x1b[B", b"\x1bOB"),
        Key::Right => cursor_key(b"\x1b[C", b"\x1bOC"),
        Key::Left => cursor_key(b"\x1b[D", b"\x1bOD"),
        Key::Home => cursor_key(b"\x1b[H", b"\x1bOH"),
        Key::End => cursor_key(b"\x1b[F", b"\x1bOF"),
        Key::Insert => b"\x1b[2~",
        Key::Delete => b"\x1b[3~",
        Key::PageUp => b"\x1b[5~",
        Key::PageDown => b"\x1b[6~",
        Key::Function(1) => b"\x1bOP",
        Key::Function(2) => b"\x1bOQ",
        Key::Function(3) => b"\x1bOR",
        Key::Function(4) => b"\x1bOS",
        Key::Function(5) => b"\x1b[15~",
        Key::Function(7) => b"\x1b[18~",
        Key::Function(8) => b"\x1b[19~",
        Key::Function(9) => b"\x1b[20~",
        Key::Function(10) => b"\x1b[21~",
        Key::Function(11) => b"\x1b[23~",
        Key::Function(12) => b"\x1b[24~",
        _ => return None,
    };
    Some(bytes.to_vec())
}

fn grid_row(screen: &vt100::Screen, row: u16, cols: u16, cursor: Option<(u16, u16)>) -> GridRow {
    let base_ink = theme::ink();
    let base_fill = theme::canvas();
    let mut line = String::with_capacity(usize::from(cols));
    let mut spans = Vec::new();
    let mut wide = Vec::new();
    let mut fills = Vec::new();
    for col in 0..cols {
        let Some(cell) = screen.cell(row, col) else {
            line.push(' ');
            continue;
        };
        let foreground = cell_color(cell.fgcolor(), base_ink);
        let background = cell_color(cell.bgcolor(), base_fill);
        let (foreground, background) = if cell.inverse() || cursor == Some((row, col)) {
            (background, foreground)
        } else {
            (foreground, background)
        };
        if background != base_fill {
            fills.push((col, background));
        }
        let contents = cell.contents();
        if cell.is_wide_continuation() || contents.is_empty() {
            line.push(' ');
        } else if cell.is_wide() {
            line.push(' ');
            wide.push((col, contents, foreground));
        } else {
            let start = line.len();
            line.push_str(contents);
            let mut span = Span::new(start, line.len()).color(foreground);
            if cell.bold() {
                span = span.bold();
            }
            if cell.italic() {
                span = span.italic();
            }
            if cell.underline() {
                span = span.underline(foreground);
            }
            spans.push(span);
        }
    }
    let mut result = GridRow::new(line, spans, CODE_SM).wraps(screen.row_wrapped(row));
    for (col, contents, foreground) in wide {
        result = result.cell(col, 2, contents, CODE_SM, foreground);
    }
    for (col, background) in fills {
        result = result.fill(col, col + 1, background);
    }
    result
}

fn cell_color(color: vt100::Color, default: Color) -> Color {
    const ANSI: [[u8; 3]; 16] = [
        [0, 0, 0],
        [205, 49, 49],
        [13, 139, 60],
        [171, 114, 0],
        [36, 114, 200],
        [160, 66, 170],
        [10, 132, 147],
        [229, 229, 229],
        [102, 102, 102],
        [241, 76, 76],
        [35, 209, 139],
        [244, 194, 70],
        [69, 139, 219],
        [214, 112, 214],
        [41, 184, 219],
        [255, 255, 255],
    ];
    let [r, g, b] = match color {
        vt100::Color::Default => return default,
        vt100::Color::Rgb(r, g, b) => [r, g, b],
        vt100::Color::Idx(index @ 0..=15) => ANSI[usize::from(index)],
        vt100::Color::Idx(index @ 16..=231) => {
            let index = index - 16;
            let levels = [0, 95, 135, 175, 215, 255];
            [index / 36, index / 6 % 6, index % 6].map(|channel| levels[usize::from(channel)])
        }
        vt100::Color::Idx(index) => [8 + 10 * (index - 232); 3],
    };
    let decode = |value| rune::color::srgb_decode(f32::from(value) / 255.0);
    Color {
        r: decode(r),
        g: decode(g),
        b: decode(b),
        a: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn wait_for(terminal: &mut Terminal, ready: impl Fn(&Terminal) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            terminal.read_output();
            if ready(terminal) {
                break;
            }
            assert!(Instant::now() < deadline, "agent terminal did not respond");
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    #[cfg(unix)]
    fn coding_agent_session_accepts_input_and_keeps_output_after_stop() {
        let dir = tempfile::tempdir().unwrap();
        let mut terminal = Terminal::launch(Path::new("/bin/cat"), dir.path(), None).unwrap();
        terminal.resize(340.0, 300.0, CELL_WIDTH);
        terminal.send_line("Coderune PTY check\x07").unwrap();
        wait_for(&mut terminal, |terminal| {
            terminal
                .screen
                .screen()
                .contents()
                .matches("Coderune PTY check")
                .count()
                == 2
        });
        terminal.stop().unwrap();
        wait_for(&mut terminal, |terminal| !terminal.is_running());
        assert!(terminal
            .screen
            .screen()
            .contents()
            .contains("Coderune PTY check"));
    }

    #[test]
    #[cfg(windows)]
    fn coding_agent_session_runs_batch_launchers_with_spaces_and_ampersands() {
        let dir = tempfile::tempdir().unwrap();
        let launcher = dir.path().join("agent & helper.cmd");
        std::fs::write(&launcher, "@echo off\r\necho Coderune Windows shim\r\n").unwrap();
        let mut terminal = Terminal::launch(&launcher, dir.path(), None).unwrap();
        wait_for(&mut terminal, |terminal| {
            terminal
                .screen
                .screen()
                .contents()
                .contains("Coderune Windows shim")
        });
        wait_for(&mut terminal, |terminal| !terminal.is_running());
    }
}
