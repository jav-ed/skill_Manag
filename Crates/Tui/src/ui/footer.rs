//! The help line at the bottom and the full help overlay.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::Screen;
use crate::binding::{
    ALL, BACK, BACK_HELP, Binding, CONFIRM_HELP, DELETE, DOWN, FILTER, GIT, HELP, ISSUES,
    MENU_HELP, MENU_QUIT, OPEN, QUIT, RESULTS_HELP, RUN_ADD, RUN_DELETE, RUN_INIT, RUN_PUSH,
    RUN_SYNC, SCROLL, SELECT_COMMON, SYNC, TOGGLE, UP,
};
use crate::hit::{HitMap, Target};
use crate::items::Mode;
use crate::num::to_u16;
use crate::screens::{History, HistoryPhase, Phase, Place, Setup, Stage, Step, Work};
use crate::theme;

/// A key as written, and what it does.
type Hint = (&'static str, &'static str);

fn hints(bindings: &[&Binding]) -> Vec<Hint> {
    bindings.iter().map(|b| (b.label, b.help)).collect()
}

/// The action keys that differ by screen.
fn mode_keys(mode: Mode) -> Vec<&'static Binding> {
    match mode {
        Mode::Sync => vec![&RUN_SYNC],
        Mode::Push => vec![&RUN_PUSH],
        Mode::Delete => vec![&RUN_DELETE],
        Mode::List => vec![&SYNC, &DELETE],
        Mode::Add => vec![&RUN_ADD],
        Mode::Init => vec![&GIT, &RUN_INIT],
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
                let mut keys: Vec<&Binding> = vec![&TOGGLE, &ALL, &FILTER];
                keys.extend(mode_keys(work.mode));
                if !work.issues.is_empty() {
                    keys.push(&ISSUES);
                }
                keys.extend([&HELP, &BACK]);
                hints(&keys)
            }
            Phase::Confirm(_) => hints(CONFIRM_HELP),
            Phase::Done(_) => hints(RESULTS_HELP),
            Phase::Loading | Phase::Failed(_) | Phase::Planning(_) | Phase::Running { .. } => {
                hints(BACK_HELP)
            }
        },
    }
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

/// Every key of the current screen.
fn full(screen: &Screen) -> Vec<Hint> {
    match screen {
        Screen::Menu(_) => hints(MENU_HELP),
        Screen::Setup(setup) => {
            let mut keys = setup_hints(setup);
            keys.push(("ctrl+c", "quit"));
            keys
        }
        Screen::History(history) => {
            let mut keys = history_hints(history);
            keys.push(("ctrl+c", "quit"));
            keys
        }
        Screen::Place(place) => {
            let mut keys = place_hints(place);
            keys.push(("ctrl+c", "quit"));
            keys
        }
        Screen::Work(work) => work_full(work),
    }
}

fn work_full(work: &Work) -> Vec<Hint> {
    match &work.phase {
        Phase::Select => {
            let mut keys = SELECT_COMMON.to_vec();
            keys.extend(mode_keys(work.mode));
            if !work.issues.is_empty() {
                keys.push(&ISSUES);
            }
            keys.extend([&HELP, &BACK, &QUIT]);
            hints(&keys)
        }
        Phase::Confirm(_) => hints(CONFIRM_HELP),
        Phase::Done(_) => hints(RESULTS_HELP),
        _ => hints(BACK_HELP),
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
        && work.filtering
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

pub(super) fn overlay(frame: &mut Frame, hits: &mut HitMap, screen: &Screen) {
    let full_area = frame.area();
    hits.push(full_area, Target::Dismiss);
    let keys = full(screen);
    let height = to_u16(keys.len() + 2).min(full_area.height);
    let width = 44.min(full_area.width);
    let rect = Rect::new(
        full_area.x + (full_area.width - width) / 2,
        full_area.y + (full_area.height - height) / 2,
        width,
        height,
    );
    hits.push(rect, Target::Overlay);
    frame.render_widget(Clear, rect);
    let lines: Vec<Line> = keys
        .into_iter()
        .map(|(label, help)| {
            Line::from(vec![
                Span::styled(format!("{label:<10}"), theme::accent()),
                Span::raw(help),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" keys ")),
        rect,
    );
}
