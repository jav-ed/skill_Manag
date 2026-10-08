//! Drawing. Every clickable region is recorded in the hit map while it is drawn.

mod boxed;
mod dialog;
mod footer;
mod header;
mod issues;
mod link;
mod menu;
mod page;
mod picker;
mod results;
mod select;
mod setup;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::app::{App, Screen};
use crate::screens::{Phase, Work};

pub(crate) fn draw(frame: &mut Frame, app: &mut App) {
    app.hits.clear();
    let name = app.screen_name();
    let [head, _gap, body, foot] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let App {
        screen,
        hits,
        hover,
        tick,
        help,
        quit_warned,
        ..
    } = app;
    header::draw(frame, hits, *hover, name, head);
    match screen {
        Screen::Menu(menu_state) => menu::draw(frame, hits, menu_state, body),
        Screen::Work(work) => draw_work(frame, hits, *hover, *tick, work, body),
        Screen::Setup(setup_state) => setup::draw(frame, hits, *hover, setup_state, body),
    }
    let notice = quit_warned.then_some("A job is writing. Press ctrl+c again to quit anyway.");
    footer::draw(frame, screen, notice, foot);
    if *help {
        footer::overlay(frame, hits, screen);
    }
}

fn draw_work(
    frame: &mut Frame,
    hits: &mut crate::hit::HitMap,
    hover: Option<crate::hit::Target>,
    tick: usize,
    work: &mut Work,
    area: ratatui::layout::Rect,
) {
    match &work.phase {
        Phase::Loading => page::loading(frame, tick, area),
        Phase::Failed(message) => page::failed(frame, message, area),
        Phase::Select => {
            select::draw(frame, hits, work, area);
            if work.issues_open {
                issues::draw(frame, hits, &work.issues, area);
            }
        }
        Phase::Planning(kind) => page::planning(frame, *kind, tick, area),
        Phase::Confirm(_) => {
            select::draw(frame, hits, work, area);
            if let Phase::Confirm(pending) = &work.phase {
                dialog::draw(frame, hits, hover, pending, area);
            }
        }
        Phase::Running { kind, done, total } => page::running(frame, *kind, *done, *total, area),
        Phase::Done(_) => {
            let details = work.details;
            let mut scroll = work.scroll;
            if let Phase::Done(results) = &work.phase {
                results::draw(frame, results, &mut scroll, details, area);
            }
            work.scroll = scroll;
        }
    }
}
