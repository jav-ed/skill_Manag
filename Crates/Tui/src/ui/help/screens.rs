//! The help of the menu, the history, the Override page, the folder picker and the setup wizard.

use super::{Topic, general, topic};
use crate::screens::{History, HistoryPhase, Local, LocalPhase, Place, Setup, Stage, Step};

pub(in crate::ui) fn menu_topics() -> Vec<Topic> {
    vec![
        topic(
            "The menu",
            vec![
                (
                    "↑/k  ↓/j",
                    "move over the entries. The box under the list says what the entry under the cursor does.",
                ),
                ("enter", "open the entry. A click opens it too."),
            ],
        ),
        general(&[("q", "quit")]),
    ]
}

pub(in crate::ui) fn history_topics(history: &History) -> Vec<Topic> {
    let keys = match history.phase {
        HistoryPhase::List => vec![
            ("↑/k  ↓/j", "move over the runs, newest first."),
            (
                "enter",
                "undo the run under the cursor, after a question. Undoing is a run of its own, so you can undo the undo.",
            ),
            ("esc", "back to the menu."),
        ],
        HistoryPhase::Confirm(_) => vec![
            ("y  enter", "put the old copies back."),
            ("n  esc", "cancel. Nothing is changed."),
        ],
        HistoryPhase::Done(_) => vec![("↑/↓", "scroll."), ("q", "back to the menu.")],
        HistoryPhase::Loading
        | HistoryPhase::Failed(_)
        | HistoryPhase::Planning
        | HistoryPhase::Running { .. } => vec![("q", "back, when nothing is being written.")],
    };
    vec![topic("History", keys), general(&[])]
}

pub(in crate::ui) fn local_topics(local: &Local) -> Vec<Topic> {
    let keys = match local.phase {
        LocalPhase::List => vec![
            ("↑/k  ↓/j", "move over the skills of the project."),
            (
                "space",
                "tick or untick an edited skill. Only edited skills can be replaced: the others are the vault's own, older, or not in the vault.",
            ),
            ("a", "tick every edited skill, or clear them."),
            (
                "s",
                "also update the other projects that have the skills, after the vault has the new copies.",
            ),
            (
                "enter",
                "replace the vault's copy of the ticked skills (or of the skill under the cursor when none is ticked), after a question. The old copy is kept, so History can undo it.",
            ),
            ("esc", "back to the menu."),
        ],
        LocalPhase::Pick => vec![
            ("↑/k  ↓/j", "move over the projects."),
            ("enter", "look at the project under the cursor."),
            ("esc", "back to the menu."),
        ],
        LocalPhase::Confirm(_) => vec![
            ("y  enter", "replace the vault's copies."),
            ("n  esc", "cancel. Nothing is changed."),
        ],
        LocalPhase::Done(_) => vec![("↑/↓", "scroll."), ("q", "back to the menu.")],
        LocalPhase::Loading
        | LocalPhase::Failed(_)
        | LocalPhase::Planning
        | LocalPhase::Running { .. } => {
            vec![("q", "back, when nothing is being written.")]
        }
    };
    vec![topic("Override", keys), general(&[])]
}

pub(in crate::ui) fn place_topics(place: &Place) -> Vec<Topic> {
    let keys = match place.stage {
        Stage::Folder => vec![
            ("↑/k  ↓/j", "move over the folders."),
            ("l", "open the folder under the cursor."),
            ("h", "go to the parent folder."),
            ("enter", "choose the folder you are in."),
            (".", "show or hide hidden folders."),
            ("esc", "back to the menu."),
        ],
        Stage::Name => vec![
            ("enter", "continue with this name."),
            ("esc", "back to the folders."),
        ],
    };
    vec![topic("Choose a folder", keys), general(&[])]
}

pub(in crate::ui) fn setup_topics(setup: &Setup) -> Vec<Topic> {
    let folder = || {
        vec![
            ("↑/k  ↓/j", "move over the folders."),
            ("l", "open the folder under the cursor."),
            ("h", "go to the parent folder."),
            ("enter", "choose the folder you are in."),
            (".", "show or hide hidden folders."),
            ("esc", "one step back."),
        ]
    };
    let (title, keys) = match setup.step {
        Step::Vault => (
            "The vault: the git folder with your master skills",
            folder(),
        ),
        Step::Root => (
            "The root: the folder that holds all your projects",
            folder(),
        ),
        Step::Mandatory => (
            "The mandatory skills",
            vec![
                (
                    "space",
                    "tick or untick a skill. Ticked ones go into every project on a push.",
                ),
                ("a", "tick all, or untick all."),
                ("enter", "continue."),
                ("esc", "one step back."),
            ],
        ),
        Step::Save => (
            "Save",
            vec![
                ("y", "save the vault and the root."),
                ("n", "cancel."),
                ("esc", "one step back."),
            ],
        ),
        Step::Saved(_) => ("Saved", vec![("enter", "done.")]),
    };
    vec![topic(title, keys), general(&[("q", "leave the setup.")])]
}
