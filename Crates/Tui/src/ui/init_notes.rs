//! What the init page says about the switches that are off, next to the row count.

use crate::items::Mode;
use crate::screens::Work;

/// `   ·   no AGENTS.md` and `   ·   no scaffold` for the parts of a new project that are switched off.
pub(super) fn status_note(work: &Work) -> String {
    let Some(project) = work.project.as_ref().filter(|_| work.mode == Mode::Init) else {
        return String::new();
    };
    let mut note = String::new();
    if !project.agents {
        note.push_str("   ·   no AGENTS.md");
    }
    if !project.scaffold {
        note.push_str("   ·   no scaffold");
    }
    note
}
