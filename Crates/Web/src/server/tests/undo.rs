//! Undoing, and the ways a write can be held back or go partly wrong.

use serde_json::json;

use super::Server;
use super::flow::{apply, plan};
use crate::server::jobs;
use crate::server::plans::{Plans, Stored};
use crate::server::state::lock;

const CODING_ONE: &str = "projects/one/.agents/skills/coding/SKILL.md";
const CODING_TWO: &str = "projects/two/.agents/skills/coding/SKILL.md";

#[tokio::test]
async fn push_creates_the_mandatory_skill_and_undo_takes_it_away() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let made = plan(&server, &cookie, "push", &["tmux"]).await;
    assert_eq!(made["create"], 3, "{made}");

    let done = apply(&server, &cookie, &made).await;

    assert_eq!(done["failed"], 0, "{done}");
    assert!(
        server
            .world
            .exists("projects/three/.agents/skills/tmux/SKILL.md")
    );
    let backup = done["backup"]
        .as_str()
        .expect("a created skill is noted so it can be undone")
        .to_string();
    let (status, undo) = server
        .json(&cookie, "/api/undo-plan", &json!({ "run": backup }))
        .await;
    assert_eq!(status, 200, "{undo}");
    assert_eq!(undo["actionable"], 3);
    assert!(
        undo["lines"]
            .as_array()
            .unwrap()
            .iter()
            .all(|l| l["step"] == "remove")
    );
    let undone = apply(&server, &cookie, &undo).await;
    assert_eq!(undone["state"], "done", "{undone}");
    assert!(!server.world.exists("projects/three/.agents/skills/tmux"));
}

#[tokio::test]
async fn an_undo_brings_the_old_copy_back_and_can_itself_be_undone() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let made = plan(&server, &cookie, "sync", &["coding"]).await;
    let done = apply(&server, &cookie, &made).await;
    let run = done["backup"].as_str().unwrap().to_string();
    let history = server.page(&cookie, "/history").await;
    assert!(
        history.contains(&format!("data-run=\"{run}\"")),
        "{history}"
    );

    let (_, undo) = server
        .json(&cookie, "/api/undo-plan", &json!({ "run": run }))
        .await;
    assert_eq!(
        server.world.read(CODING_ONE),
        "coding v2",
        "planning an undo changes nothing"
    );
    let undone = apply(&server, &cookie, &undo).await;

    assert_eq!(undone["state"], "done", "{undone}");
    assert_eq!(server.world.read(CODING_ONE), "coding v1");
    let redo_run = undone["backup"].as_str().expect("undoing is a run too");
    let (status, _) = server
        .json(&cookie, "/api/undo-plan", &json!({ "run": redo_run }))
        .await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn an_undo_of_a_run_that_does_not_exist_says_so() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let (status, answer) = server
        .json(
            &cookie,
            "/api/undo-plan",
            &json!({ "run": "20200101-000000-0000" }),
        )
        .await;
    assert_eq!(status, 422, "{answer}");
    assert!(answer["error"].as_str().unwrap().len() > 3, "{answer}");
}

#[tokio::test]
async fn a_folder_edited_after_the_plan_is_not_overwritten() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let made = plan(&server, &cookie, "sync", &["coding"]).await;
    // Somebody works in project one while the plan waits.
    server
        .world
        .project_file("one/.agents/skills/coding/notes.md", "my notes");

    let done = apply(&server, &cookie, &made).await;

    assert_eq!(done["state"], "done");
    assert_eq!(done["failed"], 1, "{done}");
    assert_eq!(
        server
            .world
            .read("projects/one/.agents/skills/coding/notes.md"),
        "my notes"
    );
    assert_eq!(
        server.world.read(CODING_ONE),
        "coding v1",
        "the edited folder was left alone"
    );
    assert_eq!(
        server.world.read(CODING_TWO),
        "coding v2",
        "the other one went ahead"
    );
}

#[tokio::test]
async fn a_second_change_waits_for_the_first_and_its_plan_is_kept() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let made = plan(&server, &cookie, "sync", &["coding"]).await;
    // Pretend a change is running.
    let running = jobs::start(&server.state.jobs, "busy".to_string(), 1).unwrap();

    let (status, answer) = server
        .json(&cookie, "/api/apply", &json!({ "plan": made["plan"] }))
        .await;

    assert_eq!(status, 409, "{answer}");
    assert_eq!(server.world.read(CODING_ONE), "coding v1");
    lock(&server.state.jobs).finish("busy", Err("done".to_string()));
    drop(running);
    let done = apply(&server, &cookie, &made).await;
    assert_eq!(
        done["state"], "done",
        "the plan survived the refusal: {done}"
    );
}

#[test]
fn plans_are_single_use_and_only_the_newest_few_are_kept() {
    let mut plans = Plans::default();
    let undo = |n: usize| Stored::Undo { run: n.to_string() };
    for n in 0..9 {
        plans.put(format!("id{n}"), undo(n));
    }
    assert_eq!(plans.len(), 8);
    assert!(plans.take("id0").is_none(), "the oldest made room");
    assert!(plans.take("id8").is_some());
    assert!(plans.take("id8").is_none(), "taken once");
}
