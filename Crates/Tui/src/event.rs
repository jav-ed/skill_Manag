//! One channel feeds the main loop: the terminal input thread and the job threads.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::backend::InputSource;
use crate::input::Input;
use crate::results::Results;
use crate::session::Session;

/// What a background job reports.
pub(crate) enum Job {
    /// The scan finished, or failed with a message.
    Loaded(Box<Result<Session, String>>),
    /// A run advanced.
    Progress {
        done: usize,
        total: usize,
    },
    Finished(Box<Results>),
    Failed(String),
}

pub(crate) enum Event {
    Term(Input),
    Job(Job),
}

/// Reads terminal events on its own thread. It polls with a short timeout and checks a flag, so it
/// can be joined and a leaked blocking read can never steal keystrokes after the UI has ended.
pub(crate) struct InputThread {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl InputThread {
    pub(crate) fn spawn(tx: Sender<Event>, source: InputSource) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                match source.poll(Duration::from_millis(50)) {
                    Ok(Some(input)) => {
                        if tx.send(Event::Term(input)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for InputThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            drop(handle.join());
        }
    }
}
