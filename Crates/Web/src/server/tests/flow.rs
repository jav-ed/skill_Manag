//! Changing things from the browser: plan, apply, watch, undo. And that nothing changes unasked.

use serde_json::json;

use super::Server;

const CODING_ONE: &str = "projects/one/.agents/skills/coding/SKILL.md";
const CODING_TWO: &str = "projects/two/.agents/skills/coding/SKILL.md";

pub(super) async fn plan(
    server: &Server,
    cookie: &str,
    kind: &str,
    skills: &[&str],
) -> serde_json::Value {
    let (status, body) = server
        .json(
            cookie,
            "/api/plan",
            &json!({ "kind": kind, "skills": skills }),
        )
        .await;
    assert_eq!(status, 200, "{body}");
    body
}

pub(super) async fn apply(
    server: &Server,
    cookie: &str,
    plan: &serde_json::Value,
) -> serde_json::Value {
    let (status, started) = server
        .json(cookie, "/api/apply", &json!({ "plan": plan["plan"] }))
        .await;
    assert_eq!(status, 200, "{started}");
    server
        .finish(cookie, started["job"].as_str().unwrap())
        .await
}

#[tokio::test]
async fn a_plan_shows_what_would_be_written_and_writes_nothing() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let made = plan(&server, &cookie, "sync", &["coding"]).await;

    assert_eq!(made["kind"], "sync");
    assert_eq!(made["update"], 2);
    assert_eq!(made["create"], 0);
    assert_eq!(made["projects"], 2);
    let rows = made["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|r| r["action"] == "update" && r["skill"] == "coding")
    );
    assert_eq!(
        server.world.read(CODING_ONE),
        "coding v1",
        "a plan is only a plan"
    );
}

#[tokio::test]
async fn applying_a_plan_writes_exactly_it_and_keeps_a_backup() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let made = plan(&server, &cookie, "sync", &["coding"]).await;

    let done = apply(&server, &cookie, &made).await;

    assert_eq!(done["state"], "done", "{done}");
    assert_eq!(done["failed"], 0);
    assert_eq!(done["lines"].as_array().unwrap().len(), 2);
    assert!(
        done["lines"]
            .as_array()
            .unwrap()
            .iter()
            .all(|l| l["outcome"] == "updated")
    );
    assert!(
        done["backup"].is_string(),
        "the old copies were kept: {done}"
    );
    assert_eq!(server.world.read(CODING_ONE), "coding v2");
    assert_eq!(server.world.read(CODING_TWO), "coding v2");
    assert_eq!(
        server
            .world
            .read("projects/one/.agents/skills/astro/SKILL.md"),
        "astro v1",
        "astro was not in the plan"
    );
}

#[tokio::test]
async fn the_overview_looks_at_the_disk_again_after_a_write() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let before = server.page(&cookie, "/").await;
    assert!(before.contains("coding outdated"), "{before}");

    let made = plan(&server, &cookie, "sync", &["coding"]).await;
    apply(&server, &cookie, &made).await;

    let after = server.page(&cookie, "/").await;
    assert!(!after.contains("coding outdated"), "{after}");
}

#[tokio::test]
async fn a_plan_can_be_applied_once() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let made = plan(&server, &cookie, "sync", &["coding"]).await;
    apply(&server, &cookie, &made).await;

    let (status, body) = server
        .json(&cookie, "/api/apply", &json!({ "plan": made["plan"] }))
        .await;

    assert_eq!(status, 404, "{body}");
    assert!(body["error"].as_str().unwrap().contains("gone"), "{body}");
}

#[tokio::test]
async fn an_unknown_plan_is_not_applied() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let (status, _) = server
        .json(
            &cookie,
            "/api/apply",
            &json!({ "plan": "0123456789abcdef" }),
        )
        .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn a_server_that_only_looks_refuses_every_change_and_still_rescans() {
    let server = Server::new(false);
    let cookie = server.login().await;

    for (path, body) in [
        ("/api/plan", json!({ "kind": "sync", "skills": ["coding"] })),
        ("/api/apply", json!({ "plan": "x" })),
        ("/api/undo-plan", json!({ "run": "x" })),
    ] {
        let (status, answer) = server.json(&cookie, path, &body).await;
        assert_eq!(status, 403, "{path}: {answer}");
        assert!(
            answer["error"].as_str().unwrap().contains("--allow-write"),
            "{answer}"
        );
    }
    assert_eq!(server.world.read(CODING_ONE), "coding v1");
    let (status, rescanned) = server.json(&cookie, "/api/rescan", &json!({})).await;
    assert_eq!(status, 200);
    assert_eq!(rescanned["projects"], 3);
}

#[tokio::test]
async fn bad_requests_get_a_sentence_not_a_crash() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let cases = [
        (
            json!({ "kind": "delete", "skills": ["coding"] }),
            400,
            "not something to plan",
        ),
        (
            json!({ "kind": "sync", "skills": [] }),
            400,
            "at least one skill",
        ),
        (json!({ "kind": "sync", "skills": ["ghost"] }), 422, "ghost"),
        (
            json!({ "kind": "sync", "skills": ["coding"], "project": "/nowhere" }),
            422,
            "not a project",
        ),
        (
            json!({ "kind": "push", "skills": ["astro"] }),
            422,
            "not a mandatory skill",
        ),
    ];
    for (body, expected, words) in cases {
        let (status, answer) = server.json(&cookie, "/api/plan", &body).await;
        assert_eq!(status, expected, "{body}: {answer}");
        assert!(
            answer["error"].as_str().unwrap().contains(words),
            "{body}: {answer}"
        );
    }
    let response = server
        .post(&cookie, "/api/plan", &json!("not an object"))
        .await;
    assert_eq!(
        response.status(),
        422,
        "a body of the wrong shape is refused by the parser"
    );
}

#[tokio::test]
async fn a_plan_for_one_project_leaves_the_other_alone() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let two = server.world.root().join("two");
    let (status, made) = server
        .json(
            &cookie,
            "/api/plan",
            &json!({ "kind": "sync", "skills": ["coding"], "project": two }),
        )
        .await;
    assert_eq!(status, 200, "{made}");

    apply(&server, &cookie, &made).await;

    assert_eq!(server.world.read(CODING_TWO), "coding v2");
    assert_eq!(server.world.read(CODING_ONE), "coding v1");
}

#[tokio::test]
async fn sync_never_adds_a_skill_a_project_lacks() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let made = plan(&server, &cookie, "sync", &["tmux"]).await;

    assert_eq!(made["rows"].as_array().unwrap().len(), 0);
    assert_eq!(made["create"], 0);
    let done = apply(&server, &cookie, &made).await;
    assert_eq!(done["state"], "done");
    assert!(!server.world.exists("projects/one/.agents/skills/tmux"));
}

#[tokio::test]
async fn a_job_that_does_not_exist_is_404() {
    let server = Server::new(true);
    let cookie = server.login().await;
    assert_eq!(server.get(&cookie, "/api/job/nope").await.status(), 404);
}

#[tokio::test]
async fn a_plan_looks_at_the_disk_now_not_at_the_page_that_was_open() {
    let server = Server::new(true);
    let cookie = server.login().await;
    server.page(&cookie, "/").await;
    // After the page was made, a skill is added to the vault and a project gets it.
    server
        .world
        .vault_file("fresh/SKILL.md", "fresh v2")
        .commit_vault();
    server
        .world
        .project_file("one/.agents/skills/fresh/SKILL.md", "fresh v1");

    let (status, made) = server
        .json(
            &cookie,
            "/api/plan",
            &json!({ "kind": "sync", "skills": ["fresh"] }),
        )
        .await;

    assert_eq!(status, 200, "{made}");
    assert_eq!(made["update"], 1);
}

#[tokio::test]
async fn an_update_row_carries_its_diff_so_the_person_sees_the_lines_first() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let made = plan(&server, &cookie, "sync", &["coding"]).await;

    let row = &made["rows"][0];
    let file = &row["files"][0];
    assert_eq!(file["path"], "SKILL.md");
    assert_eq!(file["kind"], "changed");
    let text = file["text"].as_str().unwrap();
    assert!(
        text.contains("-coding v1") && text.contains("+coding v2"),
        "{text}"
    );
    assert_eq!(file["added"], 1);
    assert_eq!(file["removed"], 1);
}

#[tokio::test]
async fn a_new_skill_lists_its_files_as_added_and_an_unchanged_row_has_none() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let made = plan(&server, &cookie, "push", &["tmux"]).await;

    let row = &made["rows"][0];
    assert_eq!(row["action"], "create");
    assert!(row["files"].as_array().is_some(), "{row}");
}

#[test]
fn a_long_diff_is_cut_with_a_note_saying_how_much() {
    let long = "+line\n".repeat(400);
    let (shown, note) = crate::server::api::cut_for_tests(&long);
    assert_eq!(shown.lines().count(), 300);
    assert_eq!(note.as_deref(), Some("100 more lines are not shown"));
    let (whole, cut) = crate::server::api::cut_for_tests("+one\n+two\n");
    assert_eq!(whole, "+one\n+two\n");
    assert!(cut.is_none());
}
