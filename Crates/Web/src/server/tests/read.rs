//! The JSON the pages read. This is the contract between the server and the web interface: the names and
//! the shapes below are what `Ui/` depends on.

use axum::http::header;
use serde_json::{Value, json};

use super::{Server, body_bytes};

async fn read(server: &Server, cookie: &str, path: &str) -> Value {
    let response = server.get(cookie, path).await;
    assert_eq!(response.status(), 200, "{path}");
    assert!(
        response.headers()[header::CONTENT_TYPE]
            .to_str()
            .unwrap()
            .starts_with("application/json"),
        "{path}"
    );
    serde_json::from_slice(&body_bytes(response).await).unwrap()
}

#[tokio::test]
async fn the_session_says_whether_changes_are_allowed() {
    let looking = Server::new(false);
    let cookie = looking.login().await;
    let view = read(&looking, &cookie, "/api/session").await;
    assert_eq!(view["allow_write"], false);
    assert!(view["version"].as_str().unwrap().contains('.'));

    let writing = Server::new(true);
    let cookie = writing.login().await;
    assert_eq!(
        read(&writing, &cookie, "/api/session").await["allow_write"],
        true
    );
}

#[tokio::test]
async fn the_overview_has_every_project_with_what_differs() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let view = read(&server, &cookie, "/api/overview").await;

    assert!(view["at"].as_str().unwrap().ends_with("UTC"));
    assert!(view["vault"].as_str().unwrap().ends_with("/vault"));
    assert_eq!(view["drift"], 3);
    assert_eq!(view["in_sync"], 0);
    assert_eq!(view["problems"], 0);
    let projects = view["projects"].as_array().unwrap();
    assert_eq!(projects.len(), 3);
    let one = projects.iter().find(|p| p["label"] == "one").unwrap();
    assert_eq!(one["state"], "drift");
    assert!(one["path"].as_str().unwrap().ends_with("/projects/one"));
    let outdated: Vec<&str> = one["outdated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["skill"].as_str().unwrap())
        .collect();
    assert_eq!(outdated, ["astro", "coding"]);
    let astro = &one["outdated"][0];
    assert_eq!(
        (astro["added"].as_u64(), astro["changed"].as_u64()),
        (Some(1), Some(1))
    );
    assert_eq!(one["missing"], json!(["tmux"]));
    assert_eq!(one["not_in_vault"], json!(["local-only"]));
    for field in ["missing_links", "problems", "link_problems"] {
        assert!(one[field].is_array(), "{field}");
    }
}

#[tokio::test]
async fn the_overview_counts_a_comparison_that_failed_as_a_problem() {
    let server = Server::new(false);
    let cookie = server.login().await;
    let skills = server.world.root().join("one/.agents/skills");
    std::os::unix::fs::symlink(server.world.root().join("two"), skills.join("tmux")).unwrap();

    let view = read(&server, &cookie, "/api/overview").await;

    let one = view["projects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["label"] == "one")
        .unwrap();
    assert_eq!(one["problems"][0]["name"], "tmux");
    assert!(
        one["problems"][0]["message"]
            .as_str()
            .unwrap()
            .contains("symlink")
    );
}

#[tokio::test]
async fn the_sync_skills_are_the_ones_a_project_has_and_the_vault_knows() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let view = read(&server, &cookie, "/api/skills/sync").await;

    let names: Vec<&str> = view["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["coding", "astro"],
        "top-level skills first, then groups; no local-only"
    );
    let coding = &view["rows"][0];
    assert_eq!(coding["outdated"], 2);
    assert_eq!(coding["mandatory"], true);
    assert_eq!(view["rows"][1]["group"], "web");
    let labels: Vec<&str> = view["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, ["one", "three", "two"]);
}

#[tokio::test]
async fn the_push_skills_are_the_mandatory_ones_with_how_many_projects_lack_them() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let view = read(&server, &cookie, "/api/skills/push").await;

    let rows = view["rows"].as_array().unwrap();
    let names: Vec<&str> = rows.iter().map(|r| r["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["coding", "tmux"]);
    assert_eq!(rows[1]["missing"], 3);
    assert_eq!(rows[0]["missing"], 1, "project three has no coding either");
}

#[tokio::test]
async fn the_vault_lists_every_skill_with_its_card_and_the_foreign_folders_apart() {
    let server = Server::new(false);
    server
        .world
        .vault_file(
            "coding/SKILL.md",
            "---\nname: coding\ndescription: Write careful code\n---\ncoding v2",
        )
        .commit_vault();
    server.world.vault_file("coding/notes.md", "uncommitted");
    let cookie = server.login().await;

    let view = read(&server, &cookie, "/api/vault").await;

    let skills = view["skills"].as_array().unwrap();
    let names: Vec<&str> = skills.iter().map(|s| s["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["coding", "tmux", "astro"]);
    let coding = &skills[0];
    assert_eq!(coding["description"], "Write careful code");
    assert_eq!(coding["mandatory"], true);
    assert_eq!(coding["files"], json!(["SKILL.md"]));
    assert_eq!(coding["untracked"], json!(["notes.md"]));
    let states: Vec<(&str, &str)> = coding["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["project"].as_str().unwrap(), p["state"].as_str().unwrap()))
        .collect();
    assert_eq!(
        states,
        [
            ("one", "outdated"),
            ("three", "missing"),
            ("two", "outdated")
        ]
    );
    let astro = &skills[2];
    assert_eq!(astro["group"], "web");
    assert!(
        astro["header_problem"].is_string(),
        "the standard world has no header"
    );
    assert_eq!(astro["projects"][0]["detail"], "1 added, 1 changed");
    let foreign = view["foreign"].as_array().unwrap();
    assert_eq!(foreign.len(), 1);
    assert_eq!(foreign[0]["name"], "local-only");
    assert_eq!(foreign[0]["projects"][0]["state"], "not_in_vault");
}

#[tokio::test]
async fn the_history_is_empty_first_and_lists_a_run_after_a_write() {
    let server = Server::new(true);
    let cookie = server.login().await;
    assert_eq!(
        read(&server, &cookie, "/api/history").await["runs"],
        json!([])
    );

    let (_, made) = server
        .json(
            &cookie,
            "/api/plan",
            &json!({ "kind": "sync", "skills": ["coding"] }),
        )
        .await;
    let (_, started) = server
        .json(&cookie, "/api/apply", &json!({ "plan": made["plan"] }))
        .await;
    let done = server
        .finish(&cookie, started["job"].as_str().unwrap())
        .await;

    let view = read(&server, &cookie, "/api/history").await;
    let runs = view["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["id"], done["backup"]);
    assert_eq!(runs[0]["command"], "sync");
    assert_eq!(
        (runs[0]["skills"].as_u64(), runs[0]["projects"].as_u64()),
        (Some(2), Some(2))
    );
    assert!(runs[0]["date"].as_str().unwrap().ends_with("UTC"));
}

#[tokio::test]
async fn the_doctor_and_the_settings_are_readable() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let doctor = read(&server, &cookie, "/api/doctor").await;
    for finding in doctor["findings"].as_array().unwrap() {
        assert!(["note", "warning", "error"].contains(&finding["severity"].as_str().unwrap()));
        assert!(finding["check"].is_string() && finding["message"].is_string());
    }
    let settings = read(&server, &cookie, "/api/settings").await;
    assert_eq!(settings["vault"]["source"], "flag");
    assert!(
        settings["root"]["path"]
            .as_str()
            .unwrap()
            .ends_with("/projects")
    );
    assert_eq!(settings["mandatory"], json!(["coding", "tmux"]));
    assert_eq!(settings["allow_write"], false);
}

#[tokio::test]
async fn a_vault_that_cannot_be_opened_is_an_error_with_the_reason_not_an_empty_answer() {
    let server = Server::of(skillmirror_testkit::World::new(), false);
    let cookie = server.login().await;

    let response = server.get(&cookie, "/api/overview").await;

    assert_eq!(response.status(), 500);
    let body: Value = serde_json::from_slice(&body_bytes(response).await).unwrap();
    assert!(body["error"].as_str().unwrap().contains("vault"), "{body}");
}

#[tokio::test]
async fn every_read_endpoint_needs_the_session() {
    let server = Server::new(false);
    for path in [
        "/api/session",
        "/api/overview",
        "/api/skills/sync",
        "/api/skills/push",
        "/api/vault",
        "/api/history",
        "/api/doctor",
        "/api/settings",
    ] {
        assert_eq!(
            server.send(super::visit(path)).await.status(),
            401,
            "{path}"
        );
    }
}
