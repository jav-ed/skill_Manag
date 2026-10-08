use super::{Harness, world};
use crate::app::{Effect, Screen};
use crate::hit::Target;
use crate::input::MouseKind;
use crate::screens::Phase;

#[test]
fn hovering_moves_the_menu_cursor_and_a_click_opens_the_entry() {
    let mut ui = Harness::new(world());
    ui.hover(Target::MenuItem(3));
    let Screen::Menu(menu) = &ui.app.screen else {
        panic!("menu")
    };
    assert_eq!(menu.cursor, 3);
    ui.click(Target::MenuItem(1));
    ui.wait_select();
    assert!(ui.screen().contains("Skills — 4 installed"));
}

#[test]
fn clicking_a_row_toggles_it_and_hovering_moves_the_cursor() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.hover(Target::Row(1));
    let Screen::Work(work) = &ui.app.screen else {
        panic!("work")
    };
    assert_eq!(work.view.cursor, 1);
    ui.click(Target::Row(1));
    ui.click(Target::Row(2));
    let Screen::Work(work) = &ui.app.screen else {
        panic!("work")
    };
    assert_eq!(work.selected.len(), 2);
    ui.click(Target::Row(2));
    let Screen::Work(work) = &ui.app.screen else {
        panic!("work")
    };
    assert_eq!(
        work.selected.len(),
        1,
        "a second click on the same row clears it"
    );
}

#[test]
fn the_wheel_scrolls_a_long_list_and_the_scroll_track_jumps() {
    let world = world();
    for i in 0..60 {
        world.project_file(&format!("many/.agents/skills/skill-{i:02}/SKILL.md"), "x");
    }
    let mut ui = Harness::new(world);
    ui.open("List").wait_select();
    let (x, y) = ui.center_of(Target::ListArea);
    ui.mouse(MouseKind::ScrollDown, x, y);
    let Screen::Work(work) = &ui.app.screen else {
        panic!("work")
    };
    assert_eq!(work.view.offset, 3);
    ui.click(Target::ScrollTrack);
    let Screen::Work(work) = &ui.app.screen else {
        panic!("work")
    };
    assert!(
        work.view.offset > 20,
        "the track click lands about halfway: {}",
        work.view.offset
    );
}

#[test]
fn the_header_back_arrow_returns_and_the_link_asks_the_loop_to_open_the_site() {
    let mut ui = Harness::new(world());
    ui.open("Sync").wait_select();
    ui.click(Target::HeaderBack);
    assert!(matches!(ui.app.screen, Screen::Menu(_)));
    ui.click(Target::HeaderLink);
    assert_eq!(
        ui.app.take_effects(),
        vec![Effect::OpenUrl("https://javedab.com".to_string())]
    );
}

#[test]
fn clicking_cancel_or_outside_the_dialog_closes_it_and_clicking_delete_confirms() {
    let mut ui = Harness::new(world());
    ui.open("Delete").wait_select();
    ui.click(Target::Row(0)).code_enter();
    ui.click(Target::Button(1));
    assert!(matches!(&ui.app.screen, Screen::Work(w) if matches!(w.phase, Phase::Select)));
    ui.code_enter();
    ui.click(Target::Button(0)).wait_done();
    assert!(!ui.world.exists("projects/one/.agents/skills/astro"));
}

#[test]
fn a_click_while_the_help_is_open_only_closes_the_help() {
    let mut ui = Harness::new(world());
    ui.press('?');
    assert!(ui.app.help);
    ui.mouse(MouseKind::Down(crate::input::Button::Left), 0, 23);
    assert!(!ui.app.help);
    assert!(matches!(ui.app.screen, Screen::Menu(_)));
}
