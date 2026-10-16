//! The help of the selection pages: what the keys of a page do, in topics.

use super::{Topic, general, topic};
use crate::items::Mode;
use crate::screens::{Pending, Phase, Work};

pub(in crate::ui) fn work_topics(work: &Work) -> Vec<Topic> {
    match &work.phase {
        Phase::Select => select_topics(work),
        Phase::Confirm(pending) => vec![question_topic(pending), general(&[])],
        Phase::Diff(_) => vec![
            topic(
                "The changes",
                vec![
                    ("↑/↓  pgup  pgdn", "scroll. The mouse wheel scrolls too."),
                    (
                        "esc  v",
                        "go back to the question. Nothing has been written.",
                    ),
                ],
            ),
            general(&[]),
        ],
        Phase::Done(_) => vec![
            topic(
                "The results",
                vec![
                    ("↑/↓  pgup  pgdn", "scroll."),
                    (
                        "d",
                        "switch between only the failures and one line for every project.",
                    ),
                    (
                        "q  enter",
                        "go back to the menu. History can undo what was written.",
                    ),
                ],
            ),
            general(&[]),
        ],
        Phase::Loading
        | Phase::Failed(_)
        | Phase::Planning(_)
        | Phase::Diffing
        | Phase::Running { .. } => {
            vec![general(&[("q", "go back, when nothing is being written.")])]
        }
    }
}

fn question_topic(pending: &Pending) -> Topic {
    let mut keys = vec![
        ("y  enter", "do it."),
        ("n  esc", "cancel. Nothing has been written."),
    ];
    if pending.plan.is_some() || pending.agents.is_some() {
        keys.push(("v", "see every change as a diff before you decide."));
    }
    topic("The question", keys)
}

fn select_topics(work: &Work) -> Vec<Topic> {
    let mode = work.mode;
    let mut out = vec![topic(
        "Move",
        vec![
            (
                "↑/k  ↓/j",
                "one row up or down. The mouse wheel works too, and a click puts the cursor on a row.",
            ),
            ("pgup  pgdn", "a page at a time."),
        ],
    )];
    if mode != Mode::Skills {
        out.push(topic(
            "Choose",
            vec![
                (
                    "space",
                    "tick or untick the row under the cursor. Only ticked rows are acted on.",
                ),
                (
                    "a",
                    "tick every row that is shown, or untick them all when all are ticked.",
                ),
            ],
        ));
    }
    out.push(topic(
        "Find",
        vec![(
            "/",
            "filter the rows by name while you type. enter keeps the filter, esc clears it. A row the filter hides stays ticked, but nothing acts on it.",
        )],
    ));
    if let Some(pane) = pane_topic(mode) {
        out.push(pane);
    }
    if let Some(run) = run_topic(mode) {
        out.push(run);
    }
    let mut first = Vec::new();
    if !work.issues.is_empty() {
        first.push(("i", "list what the scan could not read."));
    }
    first.push(("q  alt+←", "back to the menu."));
    out.push(general(&first));
    out
}

/// The keys of the pane beside the list, for the pages that have one.
fn pane_topic(mode: Mode) -> Option<Topic> {
    if !mode.has_pane() {
        return None;
    }
    let mut keys = Vec::new();
    if mode.has_views() {
        keys.push((
            "tab",
            "switch what the pane lists: Affected (the projects this run would change), Others (the ones already up to date) or All. A click on the name does it too.",
        ));
    }
    if mode.has_project_pane() {
        keys.push((
            "p",
            "write the projects short (parent/project) or long (the whole path below your scan root). The ↔ control in the pane does the same when you click it.",
        ));
        keys.push((
            "f",
            "filter the projects in the pane. Every word you type has to be somewhere in the path, so `zetun outreach` finds the projects under both. enter keeps it, esc clears it.",
        ));
    }
    keys.push((
        "ctrl+d  ctrl+u",
        "scroll the pane down or up. The mouse wheel over the pane scrolls it too.",
    ));
    let title = if mode == Mode::Skills {
        "The card (right side, or under the list on a narrow terminal)"
    } else {
        "The projects of the row (right side, or under the list on a narrow terminal)"
    };
    Some(topic(title, keys))
}

/// What the key that does the work of the page does.
fn run_topic(mode: Mode) -> Option<Topic> {
    let keys = match mode {
        Mode::Sync => vec![(
            "enter",
            "work out what a sync would write, then ask. Nothing changes before you confirm.",
        )],
        Mode::Push => vec![(
            "enter",
            "work out what a push would write (the mandatory skills, into every project that has a skills folder), then ask.",
        )],
        Mode::Delete => vec![(
            "enter",
            "ask, then remove the ticked skills from every project that has them. A copy is kept, so History can undo it.",
        )],
        Mode::List => vec![
            ("s", "sync the ticked rows: work out the changes, then ask."),
            (
                "d",
                "delete the ticked rows after a question. A copy is kept for undo.",
            ),
        ],
        Mode::Agents => vec![(
            "enter",
            "write the autogenerated block of AGENTS.md into the ticked projects. Only the block between the markers changes. Then ask, with the diff on v.",
        )],
        Mode::Add => vec![(
            "enter",
            "install the ticked skills into the project, after a question that says what is written.",
        )],
        Mode::Init => vec![
            (
                "g",
                "make the new project a git repository on the branch main, or not. Nothing is made when the folder is inside a repository already.",
            ),
            ("m", "write an AGENTS.md as well, or leave it out."),
            (
                "d",
                "run the doc-start script as well (docs folders, Scratch, Repos, .gitignore), or leave it out. It needs the doc-start skill ticked.",
            ),
            (
                "enter",
                "create the project with the ticked skills, after a question.",
            ),
        ],
        Mode::Skills => return None,
    };
    Some(topic("Run", keys))
}
