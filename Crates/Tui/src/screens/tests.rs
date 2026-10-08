//! Unit tests of the list window and the selection logic, without a terminal.

use skillmirror_core::scan::Target;

use super::*;
use crate::input::Key;
use crate::items::{Item, Mode};
use crate::results::Kind;

fn item(name: &str, projects: &[&str], preselected: bool) -> Item {
    Item {
        name: name.to_string(),
        detail: String::new(),
        note: None,
        targets: projects
            .iter()
            .map(|p| Target {
                project: format!("/p/{p}").into(),
                skill: name.to_string(),
                path: format!("/p/{p}/.agents/skills/{name}").into(),
            })
            .collect(),
        preselected,
    }
}

fn work(mode: Mode, items: Vec<Item>) -> Work {
    let mut work = Work::new(mode);
    work.selected = items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.preselected)
        .map(|(i, _)| i)
        .collect();
    work.items = items;
    work.phase = Phase::Select;
    work.refilter();
    work.view.viewport = 3;
    work
}

fn names(count: usize) -> Vec<Item> {
    (0..count)
        .map(|i| item(&format!("skill-{i:02}"), &["a"], false))
        .collect()
}

#[test]
fn the_cursor_stays_in_range_and_the_window_follows_it() {
    let mut work = work(Mode::Sync, names(10));
    for _ in 0..20 {
        work.on_key(Key::char('j'));
    }
    assert_eq!(work.view.cursor, 9);
    assert_eq!(work.view.offset, 7, "the window shows the last three rows");
    for _ in 0..20 {
        work.on_key(Key::char('k'));
    }
    assert_eq!((work.view.cursor, work.view.offset), (0, 0));
}

#[test]
fn the_wheel_moves_the_window_and_drags_the_cursor_along() {
    let mut work = work(Mode::Sync, names(10));
    work.view.scroll_by(3);
    assert_eq!(work.view.offset, 3);
    assert!((3..6).contains(&work.view.cursor));
    work.view.scroll_by(100);
    assert_eq!(
        work.view.offset, 7,
        "never scrolls past the last full window"
    );
}

#[test]
fn a_click_on_the_scroll_track_jumps_proportionally() {
    let mut work = work(Mode::Sync, names(10));
    work.view.jump(3, 3);
    assert_eq!(work.view.offset, 7);
    work.view.jump(0, 3);
    assert_eq!(work.view.offset, 0);
}

#[test]
fn all_toggles_only_the_visible_rows_and_flips_back() {
    let mut work = work(Mode::Sync, names(4));
    work.on_key(Key::char('a'));
    assert_eq!(work.selected.len(), 4);
    work.on_key(Key::char('a'));
    assert!(work.selected.is_empty());
}

#[test]
fn enter_with_nothing_selected_does_nothing() {
    let mut work = work(Mode::Sync, names(3));
    assert_eq!(
        work.on_key(Key::press(crate::input::Code::Enter)),
        Action::None
    );
}

#[test]
fn sync_and_push_plan_first_and_delete_asks_first() {
    let items = vec![item("a", &["x", "y"], true), item("b", &["x"], false)];
    let mut sync = work(Mode::Sync, items.clone());
    let Action::Plan(pending) = sync.on_key(Key::press(crate::input::Code::Enter)) else {
        panic!("sync should work out its plan first")
    };
    assert_eq!(
        (pending.kind, pending.targets.len(), pending.skills),
        (Kind::Sync, 2, 1)
    );
    assert!(
        pending.plan.is_none(),
        "the app makes the plan, not the page"
    );

    let mut delete = work(Mode::Delete, items);
    assert_eq!(
        delete.on_key(Key::press(crate::input::Code::Enter)),
        Action::None
    );
    assert!(matches!(delete.phase, Phase::Confirm(_)));
    delete.on_key(Key::char('n'));
    assert!(matches!(delete.phase, Phase::Select), "n cancels");
    delete.on_key(Key::press(crate::input::Code::Enter));
    let Action::Run(pending) = delete.on_key(Key::char('y')) else {
        panic!("y confirms")
    };
    assert_eq!(pending.kind, Kind::Delete);
}

#[test]
fn the_filter_narrows_the_rows_and_selection_survives_it() {
    let items = vec![
        item("coding", &["a"], true),
        item("tmux", &["a"], false),
        item("astro", &["a"], false),
    ];
    let mut work = work(Mode::List, items);
    work.on_key(Key::char('/'));
    assert!(work.filtering);
    for c in "tmx".chars() {
        work.on_key(Key::char(c));
    }
    assert_eq!(work.view.len(), 1, "fuzzy: tmx finds tmux");
    work.on_key(Key::press(crate::input::Code::Enter));
    assert!(!work.filtering);
    assert!(work.selected.contains(&0), "the hidden row stays selected");
    work.on_key(Key::press(crate::input::Code::Esc));
    assert_eq!(work.view.len(), 3, "escape clears the filter");
}

#[test]
fn q_is_text_while_filtering_and_back_otherwise() {
    let mut work = work(Mode::List, names(2));
    work.on_key(Key::char('/'));
    assert_eq!(work.on_key(Key::char('q')), Action::None);
    assert_eq!(work.filter.value(), "q");
    work.on_key(Key::press(crate::input::Code::Esc));
    assert_eq!(work.on_key(Key::char('q')), Action::Back);
}

#[test]
fn running_ignores_keys_so_a_second_enter_cannot_restart_it() {
    let mut work = work(Mode::Sync, names(2));
    work.phase = Phase::Running {
        kind: Kind::Sync,
        done: 1,
        total: 2,
    };
    assert_eq!(
        work.on_key(Key::press(crate::input::Code::Enter)),
        Action::None
    );
    assert_eq!(work.on_key(Key::char('q')), Action::None);
}

#[test]
fn plural_and_short_path_helpers_read_naturally() {
    assert_eq!(crate::num::plural(1, "project"), "1 project");
    assert_eq!(crate::num::plural(0, "project"), "0 projects");
    assert_eq!(crate::results::short_path("/home/x/code/app"), "code/app");
    assert_eq!(crate::results::short_path("/solo"), "solo");
}
