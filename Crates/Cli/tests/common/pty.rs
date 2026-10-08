//! A real pseudo terminal: portable-pty runs the binary, vt100 emulates the screen it paints.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{Child, CommandBuilder, PtySize, native_pty_system};
use skillmirror_testkit::World;

pub(crate) struct Terminal {
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    screen: Arc<Mutex<vt100::Parser>>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        drop(self.child.kill());
    }
}

impl Terminal {
    /// Starts `skillmirror <args>` with the world's isolated environment.
    pub(crate) fn spawn(world: &World, args: &[&str], rows: u16, cols: u16) -> Self {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_skillmirror"));
        cmd.env_clear();
        for (key, value) in world.env() {
            cmd.env(key, value);
        }
        cmd.env("TERM", "xterm-256color");
        cmd.args(args);
        let child = pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();
        let screen = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 0)));
        let sink = Arc::clone(&screen);
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                sink.lock().unwrap().process(buf.get(..n).unwrap());
            }
        });
        Self {
            writer,
            child,
            screen,
        }
    }

    pub(crate) fn text(&self) -> String {
        self.screen.lock().unwrap().screen().contents()
    }

    pub(crate) fn wait_for(&self, needle: &str) {
        let end = Instant::now() + Duration::from_secs(10);
        while Instant::now() < end {
            if self.text().contains(needle) {
                return;
            }
            thread::sleep(Duration::from_millis(15));
        }
        panic!("timed out waiting for {needle:?}; screen:\n{}", self.text());
    }

    pub(crate) fn send(&mut self, bytes: &str) {
        self.writer.write_all(bytes.as_bytes()).unwrap();
        self.writer.flush().unwrap();
    }

    /// A left click in SGR mouse encoding; `column` and `row` are 0-based.
    pub(crate) fn click(&mut self, column: u16, row: u16) {
        self.send(&format!("\x1b[<0;{};{}M", column + 1, row + 1));
        self.send(&format!("\x1b[<0;{};{}m", column + 1, row + 1));
    }

    /// Waits for the process to end and returns its exit code.
    pub(crate) fn exit_code(&mut self) -> u32 {
        let end = Instant::now() + Duration::from_secs(10);
        while Instant::now() < end {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status.exit_code();
            }
            thread::sleep(Duration::from_millis(15));
        }
        panic!("the process did not end; screen:\n{}", self.text());
    }

    /// User plus system CPU ticks the process has used so far.
    pub(crate) fn cpu_ticks(&self) -> u64 {
        let pid = self.child.process_id().unwrap();
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        let rest = stat.rsplit(')').next().unwrap();
        let fields: Vec<&str> = rest.split_whitespace().collect();
        fields.get(11).unwrap().parse::<u64>().unwrap()
            + fields.get(12).unwrap().parse::<u64>().unwrap()
    }
}
