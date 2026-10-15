//! The help line at the bottom: the few keys that matter right now. The full help is `help.rs`.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::Screen;
use crate::binding::{
    AGENTS_FILE, ALL, BACK, BACK_HELP, Binding, CANCEL, CONFIRM_HELP, DELETE, DOWN, FILTER, GIT,
    HELP, ISSUES, MENU_QUIT, OPEN, PANE_VIEW, QUIT, RESULTS_HELP, RUN_ADD, RUN_AGENTS, RUN_DELETE,
    RUN_INIT, RUN_PUSH, RUN_SYNC, SCROLL, SYNC, TOGGLE, UP, VIEW, YES,
};
use crate::items::Mode;
use crate::screens::{History, HistoryPhase, Pending, Phase, Place, Setup, Stage, Step};
use crate::theme;

/// A key as written, and what it does.
type Hint = (&'static str, &'static str);

fn hints(bindings: &[&Binding]) -> Vec<Hint> {
    bindings.iter().map(|b| (b.label, b.help)).collect()
}

/// The action keys that differ by screen.
fn mode_keys(mode: Mode) -> Vec<&'static Binding> {
    match mode {
        Mode::Sync => vec![&PANE_VIEW, &RUN_SYNC],
        Mode::Push => vec![&PANE_VIEW, &RUN_PUSH],
        Mode::Delete => vec![&RUN_DELETE],
        Mode::List => vec![&SYNC, &DELETE],
        Mode::Skills => Vec::new(),
        Mode::Agents => vec![&RUN_AGENTS],
        Mode::Add => vec![&RUN_ADD],
        Mode::Init => vec![&GIT, &AGENTS_FILE, &RUN_INIT],
    }
}

/// The few keys that matter right now.
fn short(screen: &Screen) -> Vec<Hint> {
    match screen {
        Screen::Menu(_) => hints(&[&UP, &DOWN, &OPEN, &HELP, &MENU_QUIT]),
        Screen::Setup(setup) => setup_hints(setup),
        Screen::History(history) => history_hints(history),
        Screen::Place(place) => place_hints(place),
        Screen::Work(work) => match &work.phase {
            Phase::Select => {
                let mut keys: Vec<&Binding> = if work.mode == Mode::Skills {
                    vec![&UP, &DOWN, &FILTER]
                } else {
                    vec![&TOGGLE, &ALL, &FILTER]
                };
                keys.extend(mode_keys(work.mode));
                if !work.issues.is_empty() {
                    keys.push(&ISSUES);
                }
                keys.extend([&HELP, &BACK]);
                hints(&keys)
            }
            Phase::Confirm(pending) => confirm_hints(pending),
            Phase::Diff(_) => vec![("↑/↓", "scroll"), ("esc", "back to the question")],
            Phase::Done(_) => hints(RESULTS_HELP),
            Phase::Loading
            | Phase::Failed(_)
            | Phase::Planning(_)
            | Phase::Diffing
            | Phase::Running { .. } => hints(BACK_HELP),
        },
    }
}

/// The keys of a question; a plan can be looked at, a delete has none.
fn confirm_hints(pending: &Pending) -> Vec<Hint> {
    let mut keys: Vec<&Binding> = vec![&YES, &CANCEL];
    if pending.plan.is_some() || pending.agents.is_some() {
        keys.push(&VIEW);
    }
    keys.push(&QUIT);
    hints(&keys)
}

fn place_hints(place: &Place) -> Vec<Hint> {
    match place.stage {
        Stage::Folder => vec![
            ("↑/↓", "move"),
            ("l", "open"),
            ("h", "parent"),
            ("enter", "choose this folder"),
            (".", "hidden"),
            ("esc", "back"),
            ("q", "leave"),
        ],
        Stage::Name => vec![("enter", "continue"), ("esc", "back")],
    }
}

fn history_hints(history: &History) -> Vec<Hint> {
    match history.phase {
        HistoryPhase::List => vec![
            ("↑/↓", "move"),
            ("enter", "undo this run"),
            ("?", "help"),
            ("esc", "back"),
        ],
        HistoryPhase::Confirm(_) => hints(CONFIRM_HELP),
        HistoryPhase::Done(_) => hints(&[&SCROLL, &BACK, &QUIT]),
        HistoryPhase::Loading
        | HistoryPhase::Failed(_)
        | HistoryPhase::Planning
        | HistoryPhase::Running { .. } => hints(BACK_HELP),
    }
}

fn setup_hints(setup: &Setup) -> Vec<Hint> {
    match setup.step {
        Step::Vault | Step::Root => vec![
            ("↑/↓", "move"),
            ("l", "open"),
            ("h", "parent"),
            ("enter", "choose this folder"),
            (".", "hidden"),
            ("esc", "back"),
            ("q", "leave"),
        ],
        Step::Mandatory => vec![
            ("space", "toggle"),
            ("a", "all"),
            ("enter", "continue"),
            ("esc", "back"),
            ("q", "leave"),
        ],
        Step::Save => vec![("y", "save"), ("n", "cancel"), ("esc", "back")],
        Step::Saved(_) => vec![("enter", "done")],
    }
}

pub(super) fn draw(frame: &mut Frame, screen: &Screen, notice: Option<&str>, area: Rect) {
    if let Some(notice) = notice {
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled(notice.to_string(), theme::warning()),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }
    if let Screen::Work(work) = screen
        && (work.filtering || work.pane.filtering)
        && matches!(work.phase, Phase::Select)
    {
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled("enter", theme::accent()),
            Span::styled(" keep filter • ", theme::muted()),
            Span::styled("esc", theme::accent()),
            Span::styled(" clear", theme::muted()),
        ]);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }
    let mut spans = vec![Span::raw(" ")];
    for (i, (label, help)) in short(screen).into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" • ", theme::muted()));
        }
        spans.push(Span::styled(label, theme::accent()));
        spans.push(Span::styled(format!(" {help}"), theme::muted()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
