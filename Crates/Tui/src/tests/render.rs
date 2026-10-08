use super::{Harness, world};
use crate::input::Code;

/// The backup run id changes with every run; the snapshot keeps its place and hides the value.
fn without_run_id(screen: &str) -> String {
    const MARK: &str = "Backup: run ";
    screen
        .lines()
        .map(|line| match line.find(MARK) {
            Some(at) => {
                let rest = &line[at + MARK.len()..];
                let end = rest.find(' ').unwrap_or(rest.len());
                format!("{}{MARK}[RUN]{}", &line[..at], &rest[end..])
            }
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_menu_lists_every_entry_with_the_detail_of_the_selected_one() {
    let mut ui = Harness::new(world());
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_sync_page_shows_what_each_skill_will_do() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_delete_dialog_names_the_damage() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press(' ').code(Code::Enter);
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_results_page_lists_failures_and_collapses_successes() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('a')
        .press('s')
        .wait_confirm()
        .press('y')
        .wait_done();
    insta::assert_snapshot!(without_run_id(&ui.screen()));
    ui.press('d');
    let detailed = ui.screen();
    assert!(
        detailed.contains("updated"),
        "d shows every project: {detailed}"
    );
}

#[test]
fn the_sync_confirmation_lists_the_files_it_would_remove() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.world
        .project_file("one/.agents/skills/astro/my_notes.md", "my notes");
    ui.world
        .project_file("two/.agents/skills/coding/old/draft.md", "draft");
    ui.code(Code::Enter).wait_confirm();
    insta::assert_snapshot!(ui.screen());
}

#[test]
fn the_filter_highlights_a_fuzzy_match_and_shows_its_prompt() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('/').type_text("cdg");
    let page = ui.screen();
    assert!(page.contains("filter: cdg"), "{page}");
    assert!(page.contains("matching \"cdg\""), "{page}");
    assert!(page.contains("coding"), "{page}");
    assert!(!page.contains("astro"), "{page}");
}

#[test]
fn the_full_help_lists_the_keys_of_the_current_screen() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('?');
    let page = ui.screen();
    assert!(page.contains("keys"), "{page}");
    assert!(page.contains("space") && page.contains("toggle"), "{page}");
    assert!(page.contains("delete"), "{page}");
}

#[test]
fn no_screen_panics_on_a_tiny_terminal() {
    for (w, h) in [(1, 1), (10, 3), (30, 6), (60, 10)] {
        for label in ["Sync", "List", "Delete", "Push"] {
            let mut ui = Harness::sized(world(), w, h);
            ui.screen();
            ui.open(label).wait_select();
            ui.screen();
            ui.press('?');
            ui.screen();
            ui.press('x');
            ui.press(' ').code(Code::Enter);
            ui.screen();
        }
    }
}

#[test]
fn the_planning_confirmation_and_running_pages_survive_a_tiny_terminal() {
    for (w, h) in [(1, 1), (10, 3), (30, 6), (60, 10), (80, 24)] {
        for label in ["Sync", "Push"] {
            let mut ui = Harness::sized(world(), w, h);
            ui.world
                .project_file("one/.agents/skills/coding/stray_a.md", "x");
            ui.world
                .project_file("one/.agents/skills/coding/stray_b.md", "x");
            ui.open(label).wait_select();
            ui.code(Code::Enter);
            ui.screen(); // planning
            ui.wait_confirm();
            ui.screen(); // the question, with files to name
            ui.press('y');
            ui.screen(); // writing
            ui.wait_done();
            ui.screen();
        }
    }
}

#[test]
fn an_empty_vault_selection_explains_itself() {
    let world = world();
    for project in ["one", "two"] {
        std::fs::remove_dir_all(world.root().join(project).join(".agents/skills/coding")).unwrap();
    }
    std::fs::remove_dir_all(world.root().join("one/.agents/skills/astro")).unwrap();
    let mut ui = Harness::new(world);
    ui.open("Sync").wait_select();
    assert!(
        ui.screen()
            .contains("No matching skills found in any project.")
    );
}

/// Whether `text` starts with the pattern, where `d` stands for one digit and any other character for itself.
fn starts_like(text: &[char], pattern: &str) -> Option<usize> {
    let mut length = 0;
    for (have, want) in text.iter().zip(pattern.chars()) {
        let fits = if want == 'd' {
            have.is_ascii_digit()
        } else {
            *have == want
        };
        if !fits {
            return None;
        }
        length += 1;
    }
    (length == pattern.chars().count()).then_some(length)
}

/// Run ids and their dates change with the clock; a snapshot keeps their place and hides the value.
fn without_times(screen: &str) -> String {
    let chars: Vec<char> = screen.chars().collect();
    let mut out = String::new();
    let mut at = 0;
    while at < chars.len() {
        let rest = &chars[at..];
        if let Some(length) = starts_like(rest, "dddd-dd-dd dd:dd:dd UTC") {
            out.push_str("YYYY-MM-DD hh:mm:ss UTC");
            at += length;
        } else if let Some(length) = starts_like(rest, "dddddddd-dddddd-ddd-") {
            // The rest of an id: the process id and the counter.
            let tail = rest[length..]
                .iter()
                .take_while(|c| c.is_ascii_digit() || **c == '-')
                .count();
            out.push_str("[RUN]");
            at += length + tail;
        } else {
            out.push(chars[at]);
            at += 1;
        }
    }
    out
}

#[test]
fn the_history_page_the_undo_question_and_the_results() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.run_confirmed();
    ui.press('q');
    ui.open("History");
    ui.wait_for("the list", |app| {
        matches!(&app.screen, crate::app::Screen::History(h) if matches!(h.phase, crate::screens::HistoryPhase::List))
    });
    insta::assert_snapshot!("history_list", without_times(&ui.screen()));

    ui.code(Code::Enter);
    ui.wait_for("the question", |app| {
        matches!(&app.screen, crate::app::Screen::History(h) if matches!(h.phase, crate::screens::HistoryPhase::Confirm(_)))
    });
    insta::assert_snapshot!("history_question", without_times(&ui.screen()));

    ui.press('y');
    ui.wait_for("the results", |app| {
        matches!(&app.screen, crate::app::Screen::History(h) if matches!(h.phase, crate::screens::HistoryPhase::Done(_)))
    });
    insta::assert_snapshot!("history_results", without_times(&ui.screen()));
}

#[test]
fn the_pages_of_add_and_init() {
    let mut ui = Harness::new(world());
    let root = ui.world.root().display().to_string();
    let plain = |screen: String| screen.replace(&root, "[ROOT]");

    ui.open("Add");
    insta::assert_snapshot!("add_folder_page", plain(ui.screen()));
    ui.press('j')
        .press('j')
        .press('j')
        .press('l')
        .code(Code::Enter);
    ui.wait_select();
    insta::assert_snapshot!("add_selection_page", plain(ui.screen()));
    ui.press('q');

    ui.open("Init");
    ui.code(Code::Enter).type_text("fresh");
    insta::assert_snapshot!("init_name_page", plain(ui.screen()));
    ui.code(Code::Enter).wait_select();
    ui.press('g').code(Code::Enter).wait_confirm();
    insta::assert_snapshot!("init_question", plain(ui.screen()));
}
