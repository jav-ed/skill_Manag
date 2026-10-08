//! Plans waiting for a go-ahead. A plan is applied exactly as it was shown, once.

use std::time::{Duration, Instant};

use skillmirror_core::backup::RunKind;
use skillmirror_core::plan::Plan;

/// How long a plan waits for its confirmation.
const LIFETIME: Duration = Duration::from_secs(15 * 60);
/// Plans kept at once; the oldest goes when a new one arrives.
const MOST: usize = 8;

pub(crate) enum Stored {
    /// A sync or push, with the plan the page showed.
    Write { kind: RunKind, plan: Plan },
    /// An undo of this run. The engine checks the disk again when it runs.
    Undo { run: String },
}

struct Item {
    id: String,
    made: Instant,
    stored: Stored,
}

#[derive(Default)]
pub(crate) struct Plans {
    items: Vec<Item>,
}

impl Plans {
    /// Keeps a plan and returns the word that names it.
    pub(crate) fn put(&mut self, id: String, stored: Stored) -> String {
        self.items.retain(|item| item.made.elapsed() < LIFETIME);
        while self.items.len() >= MOST {
            self.items.remove(0);
        }
        self.items.push(Item {
            id: id.clone(),
            made: Instant::now(),
            stored,
        });
        id
    }

    /// Takes the plan out: it cannot be applied twice. A plan that waited too long is gone.
    pub(crate) fn take(&mut self, id: &str) -> Option<Stored> {
        let at = self.items.iter().position(|item| item.id == id)?;
        let item = self.items.remove(at);
        (item.made.elapsed() < LIFETIME).then_some(item.stored)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }
}
