use super::{Harness, world};
use crate::app::Screen;
use crate::input::Code;
use crate::screens::Phase;

fn phase_name(app: &crate::app::App) -> &'static str {
    match &app.screen {
        Screen::Menu(_) => "menu",
        Screen::Setup(_) => "setup",
        Screen::Work(w) => match w.phase {
            Phase::Loading => "loading",
            Phase::Failed(_) => "failed",
            Phase::Select => "select",
            Phase::Planning(_) => "planning",
            Phase::Confirm(_) => "confirm",
            Phase::Running { .. } => "running",
            Phase::Done(_) => "done",
        },
    }
}

#[test]
fn sync_updates_installed_skills_only_and_shows_the_results() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    let page = ui.screen();
    assert!(page.contains("coding"), "{page}");
    assert!(page.contains("2 to update"), "{page}");
    assert!(page.contains("2 / 2 selected"), "{page}");
    assert!(!page.contains("tmux"), "sync never adds a skill: {page}");

    ui.run_confirmed();
    let page = ui.screen();
    assert!(page.contains("Sync results"), "{page}");
    assert!(page.contains("synced to 2 projects"), "{page}");
    assert_eq!(
        ui.world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        ui.world.read("projects/two/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
    assert_eq!(
        ui.world.read("projects/one/.agents/skills/astro/ref.md"),
        "ref"
    );
    assert!(!ui.world.exists("projects/two/.agents/skills/tmux"));
}

#[test]
fn a_second_sync_finds_everything_up_to_date_and_selects_nothing() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.run_confirmed();
    ui.press('q');
    assert_eq!(phase_name(&ui.app), "menu");
    ui.open("Sync").wait_select();
    let page = ui.screen();
    assert!(page.contains("up to date"), "{page}");
    assert!(page.contains("0 / 2 selected"), "{page}");
    ui.code(Code::Enter);
    assert_eq!(
        phase_name(&ui.app),
        "select",
        "nothing selected: enter does nothing"
    );
}

#[test]
fn push_installs_the_mandatory_skills_into_every_project_with_a_skills_directory() {
    let mut ui = Harness::new(world());
    ui.open("Push").wait_select();
    let page = ui.screen();
    assert!(
        page.contains("tmux") && page.contains("3 projects"),
        "{page}"
    );
    ui.run_confirmed();
    for project in ["one", "two", "three"] {
        assert_eq!(
            ui.world
                .read(&format!("projects/{project}/.agents/skills/tmux/SKILL.md")),
            "tmux v2"
        );
    }
    assert!(!ui.world.exists("projects/plain/.agents"));
}

#[test]
fn delete_asks_before_it_removes_and_cancel_keeps_everything() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    assert!(ui.screen().contains("0 / 3 selected"));
    ui.code(Code::Enter);
    assert_eq!(phase_name(&ui.app), "select", "nothing selected: no dialog");
    ui.press(' ').code(Code::Enter);
    assert_eq!(phase_name(&ui.app), "confirm");
    let page = ui.screen();
    assert!(
        page.contains("This will inshallah permanently remove"),
        "{page}"
    );
    ui.press('n');
    assert_eq!(phase_name(&ui.app), "select");
    assert!(ui.world.exists("projects/one/.agents/skills/astro"));

    ui.code(Code::Enter).press('y').wait_done();
    let page = ui.screen();
    assert!(
        page.contains("Delete results") && page.contains("deleted from 1 project"),
        "{page}"
    );
    assert!(!ui.world.exists("projects/one/.agents/skills/astro"));
    assert!(
        ui.world.exists("projects/one/.agents/skills/coding"),
        "other skills stay"
    );
    assert!(
        ui.world.exists("projects/one/.agents/skills/local-only"),
        "unrelated skills stay"
    );
}

#[test]
fn list_deletes_exactly_the_selected_rows_after_confirming() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    let page = ui.screen();
    assert!(page.contains("Skills — 4 installed"), "{page}");
    assert!(
        page.contains("local-only") && page.contains("not in the vault"),
        "{page}"
    );
    ui.type_text("/loc").code(Code::Enter);
    ui.press(' ');
    ui.press('d');
    assert_eq!(phase_name(&ui.app), "confirm");
    ui.press('y').wait_done();
    assert!(!ui.world.exists("projects/one/.agents/skills/local-only"));
    assert!(ui.world.exists("projects/one/.agents/skills/coding"));
}

#[test]
fn list_sync_refreshes_the_selected_row_and_skips_what_the_vault_lacks() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('a')
        .press('s')
        .wait_confirm()
        .press('y')
        .wait_done();
    let page = ui.screen();
    assert!(page.contains("local-only"), "{page}");
    assert!(
        page.contains("failed"),
        "a skill the vault lacks is reported, never touched: {page}"
    );
    assert_eq!(
        ui.world
            .read("projects/one/.agents/skills/local-only/SKILL.md"),
        "mine"
    );
    assert_eq!(
        ui.world.read("projects/one/.agents/skills/coding/SKILL.md"),
        "coding v2"
    );
}

#[test]
fn a_failing_scan_shows_the_message_and_goes_back_with_q() {
    let world = world();
    std::fs::remove_dir_all(world.vault().join(".git")).unwrap();
    let mut ui = Harness::new(world);
    ui.open("Sync")
        .wait_for("the error page", |app| phase_name(app) == "failed");
    let page = ui.screen();
    assert!(page.contains('✗'), "{page}");
    assert!(
        page.to_lowercase().contains("git"),
        "the reason is shown: {page}"
    );
    ui.press('q');
    assert_eq!(phase_name(&ui.app), "menu");
}

#[test]
fn ctrl_c_quits_from_anywhere_even_while_typing() {
    let mut ui = Harness::new(world());
    ui.open("List").wait_select();
    ui.press('/');
    ui.ctrl('c');
    assert!(ui.app.should_quit());
}

#[test]
fn q_in_the_menu_quits_and_the_results_page_goes_back_to_the_menu_entry() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.press('q');
    let Screen::Menu(menu) = &ui.app.screen else {
        panic!("back at the menu")
    };
    assert_eq!(menu.cursor, 2, "the cursor stays on Delete");
    ui.press('q');
    assert!(ui.app.should_quit());
}

/// The run folders in the isolated state directory the harness uses.
fn backup_runs(ui: &Harness) -> Vec<String> {
    let store = ui.world.path().join("home/state/skillmirror/backups");
    let mut ids: Vec<_> = std::fs::read_dir(store)
        .map(|entries| {
            entries
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    ids
}

#[test]
fn a_sync_keeps_the_old_copies_and_the_results_page_names_the_run() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();

    ui.run_confirmed();

    let ids = backup_runs(&ui);
    assert_eq!(ids.len(), 1, "{ids:?}");
    let page = ui.screen();
    assert!(page.contains(&format!("Backup: run {}", ids[0])), "{page}");
    let kept = ui
        .world
        .path()
        .join("home/state/skillmirror/backups")
        .join(&ids[0])
        .join("0/tree/SKILL.md");
    assert!(
        std::fs::read_to_string(kept).unwrap().ends_with("v1"),
        "the old copy is in the store"
    );
}

#[test]
fn a_delete_keeps_the_folder_in_the_store() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();

    ui.press(' ')
        .code(Code::Enter)
        .code(Code::Enter)
        .wait_done();

    let ids = backup_runs(&ui);
    assert_eq!(ids.len(), 1, "{ids:?}");
    assert!(
        ui.screen().contains("Backup: run "),
        "the page names the run"
    );
}

#[test]
fn a_job_that_changes_nothing_leaves_no_backup() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.run_confirmed();
    let first = backup_runs(&ui);
    ui.press('q');
    ui.open("Sync").wait_select();

    // Nothing is selected when everything is up to date, so select all to run the job anyway.
    ui.press('a').code(Code::Enter).wait_done();

    assert_eq!(backup_runs(&ui), first, "an up-to-date sync stores nothing");
}
