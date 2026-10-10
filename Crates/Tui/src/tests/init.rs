//! Init through the interface: choose a parent folder, name the project, choose skills, write.

use super::place_support::{place, selected_page, with_targets};
use super::{Harness, world};
use crate::app::Screen;
use crate::input::Code;
use crate::screens::Stage;

#[test]
fn init_makes_the_project_with_the_mandatory_skills_ticked() {
    let mut ui = Harness::new(world());
    ui.open("Init");
    assert_eq!(place(&ui).stage, Stage::Folder);
    ui.code(Code::Enter);
    assert_eq!(place(&ui).stage, Stage::Name, "the root is the parent");
    ui.type_text("fresh").code(Code::Enter).wait_select();
    // The standard vault has not the skills the built-in AGENTS.md names, so the file is switched off.
    ui.press('m');

    let page = selected_page(&mut ui);
    assert!(page.contains("Select skills for projects/fresh"), "{page}");
    assert!(
        page.contains("2 / 3 selected"),
        "coding and tmux are mandatory: {page}"
    );
    assert!(
        !ui.world.exists("projects/fresh"),
        "nothing is made before the question is answered"
    );

    ui.press('g');
    assert!(ui.screen().contains("(git repository)"));
    ui.code(Code::Enter).wait_confirm();
    let page = ui.screen();
    assert!(
        page.contains("Create projects/fresh with 2 skills?"),
        "{page}"
    );
    assert!(page.contains("It becomes a git repository."), "{page}");
    assert!(!ui.world.exists("projects/fresh"), "still nothing made");

    ui.press('y').wait_done();
    let page = ui.screen();
    assert!(page.contains("Init results"), "{page}");
    assert!(page.contains("installed in 1 project"), "{page}");
    for skill in ["coding", "tmux"] {
        assert!(
            ui.world
                .exists(&format!("projects/fresh/.agents/skills/{skill}/SKILL.md")),
            "{skill}"
        );
    }
    assert!(ui.world.exists("projects/fresh/.git"));
}

#[test]
fn init_asks_for_a_usable_name() {
    let mut ui = Harness::new(world());
    ui.open("Init");
    ui.code(Code::Enter);

    ui.code(Code::Enter);
    assert!(ui.screen().contains("Type a name"), "an empty name");
    ui.type_text("a/b").code(Code::Enter);
    assert!(ui.screen().contains("without slashes"));
    // `q` is a letter here, not the way out.
    ui.code(Code::Esc);
    assert_eq!(place(&ui).stage, Stage::Folder, "esc steps back");
    ui.code(Code::Enter).ctrl('u').type_text("quiet");
    assert_eq!(place(&ui).name.value(), "quiet");
}

#[test]
fn init_refuses_a_folder_that_is_not_empty() {
    let mut ui = Harness::new(world());
    ui.open("Init");
    ui.code(Code::Enter).type_text("one").code(Code::Enter);
    ui.wait_for(
        "the look at the folder",
        |app| matches!(&app.screen, Screen::Place(p) if !p.checking),
    );

    let page = ui.screen();
    assert!(page.contains("is not empty"), "{page}");
    assert!(
        ui.world
            .exists("projects/one/.agents/skills/coding/SKILL.md"),
        "the project is untouched"
    );
}

#[test]
fn init_takes_the_new_folder_back_when_nothing_could_be_installed() {
    let world = world();
    // Links are asked for, but there is no project to put them in when nothing got installed.
    with_targets(&world);
    // In the vault but not in git: the plan for it fails, so nothing can be installed.
    world.vault_file("fresh/SKILL.md", "untracked");
    let mut ui = Harness::new(world);
    ui.open("Init");
    ui.code(Code::Enter)
        .type_text("fresh")
        .code(Code::Enter)
        .wait_select();
    ui.press('m');
    // Untick the mandatory skills, tick the one that cannot be planned.
    ui.press('a').press('a');
    assert!(ui.screen().contains("0 / 4 selected"), "{}", ui.screen());
    ui.press('j').press('j').press(' ');

    ui.code(Code::Enter).wait_done();

    let page = ui.screen();
    assert!(page.contains("failed"), "{page}");
    assert!(
        !page.contains("Could not link"),
        "nothing was written: {page}"
    );
    assert!(
        !ui.world.exists("projects/fresh"),
        "the folder this job made is gone again"
    );
}

#[test]
fn the_question_mark_is_a_letter_in_the_name_and_help_in_the_list() {
    let mut ui = Harness::new(world());
    ui.open("Init");
    ui.code(Code::Enter);

    ui.press('?');

    assert_eq!(place(&ui).name.value(), "?");
    assert!(!ui.app.help, "typing a name does not open the help");
    ui.code(Code::Esc).press('?');
    assert!(ui.app.help, "in the folder list it does");
}

#[test]
fn an_answer_about_a_name_the_user_stepped_back_from_is_dropped() {
    let mut ui = Harness::new(world());
    ui.open("Init");
    ui.code(Code::Enter).type_text("one").code(Code::Enter);
    assert!(place(&ui).checking);

    ui.code(Code::Esc);
    assert_eq!(place(&ui).stage, Stage::Folder);
    let late = ui.next_event();
    ui.app.handle(late);

    let page = ui.screen();
    assert!(
        !page.contains("is not empty"),
        "the refusal is for a name no longer asked: {page}"
    );
}

#[test]
fn a_folder_that_got_files_after_the_question_is_never_touched() {
    let mut ui = Harness::new(world());
    ui.open("Init");
    ui.code(Code::Enter)
        .type_text("fresh")
        .code(Code::Enter)
        .wait_select();
    ui.press('m');
    ui.code(Code::Enter).wait_confirm();
    // Between the question and the yes, the user (or another program) puts a file there.
    ui.world.project_file("fresh/notes.md", "mine");

    ui.press('y');
    ui.wait_for("the end of the job", |app| {
        matches!(&app.screen, Screen::Work(w) if matches!(w.phase, crate::screens::Phase::Failed(_) | crate::screens::Phase::Done(_)))
    });

    let page = ui.screen();
    assert!(page.contains("is not empty"), "{page}");
    assert_eq!(ui.world.read("projects/fresh/notes.md"), "mine");
    assert!(
        !ui.world.exists("projects/fresh/.agents"),
        "nothing was installed into a folder that is not empty"
    );
}
