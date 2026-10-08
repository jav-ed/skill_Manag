//! The pages: what they say, and that nothing in them can become markup.

use skillmirror_testkit::World;

use super::Server;

#[tokio::test]
async fn the_overview_names_every_project_and_what_differs_in_it() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let page = server.page(&cookie, "/").await;

    assert!(page.contains("3 projects"), "{page}");
    assert!(page.contains("3 differ"), "{page}");
    for project in ["one", "two", "three"] {
        assert!(page.contains(project), "{project}");
    }
    assert!(page.contains("astro outdated"), "{page}");
    assert!(page.contains("tmux missing"), "{page}");
    assert!(page.contains("local-only not in the vault"), "{page}");
    assert!(page.contains("read-only"), "{page}");
}

#[tokio::test]
async fn the_write_mode_is_shown_in_the_header() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let page = server.page(&cookie, "/").await;
    assert!(page.contains("changes allowed"), "{page}");
    assert!(!page.contains("read-only"), "{page}");
}

#[tokio::test]
async fn names_from_the_disk_are_escaped_in_every_page() {
    let world = World::standard();
    // A skill and a project whose names are markup, as an attacker-written repository could carry.
    world.project_file(
        "<img src=x onerror=alert(1)>/.agents/skills/<script>alert(2)</script>/SKILL.md",
        "x",
    );
    let server = Server::of(world, true);
    let cookie = server.login().await;

    for path in ["/", "/sync", "/push", "/history", "/doctor", "/settings"] {
        let page = server.page(&cookie, path).await;
        assert!(!page.contains("<script>alert"), "{path}: {page}");
        assert!(!page.contains("<img src=x"), "{path}: {page}");
    }
    let overview = server.page(&cookie, "/").await;
    assert!(
        overview.contains("&lt;img src=x onerror=alert(1)&gt;"),
        "{overview}"
    );
}

#[tokio::test]
async fn the_pages_use_our_script_and_style_sheet_and_none_inline() {
    let server = Server::new(true);
    let cookie = server.login().await;

    for path in ["/", "/sync", "/push", "/history", "/doctor", "/settings"] {
        let page = server.page(&cookie, path).await;
        assert!(page.contains("href=\"/app.css\""), "{path}");
        assert!(page.contains("src=\"/app.js\""), "{path}");
        assert!(!page.contains("<style"), "{path}");
        assert!(!page.contains(" onclick="), "{path}");
        let inline_script = page.matches("<script").count() == 1 && page.contains("<script src=");
        assert!(inline_script, "{path}: only the external script may exist");
    }
}

#[tokio::test]
async fn the_sync_page_lists_skills_with_what_a_sync_would_do() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let page = server.page(&cookie, "/sync").await;

    assert!(page.contains("never adds a skill"), "{page}");
    assert!(page.contains("value=\"astro\""), "{page}");
    assert!(page.contains("value=\"coding\""), "{page}");
    assert!(page.contains("1 outdated"), "{page}");
    assert!(
        !page.contains("value=\"local-only\""),
        "a skill the vault lacks cannot be synced: {page}"
    );
    assert!(page.contains("data-action=\"plan\""), "{page}");
}

#[tokio::test]
async fn the_push_page_lists_the_mandatory_skills() {
    let server = Server::new(true);
    let cookie = server.login().await;

    let page = server.page(&cookie, "/push").await;

    assert!(
        page.contains("value=\"coding\"") && page.contains("value=\"tmux\""),
        "{page}"
    );
    assert!(!page.contains("value=\"astro\""), "{page}");
    assert!(page.contains("3 missing"), "{page}");
}

#[tokio::test]
async fn read_only_pages_have_no_checkboxes_and_no_plan_button() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let page = server.page(&cookie, "/sync").await;

    assert!(!page.contains("type=\"checkbox\""), "{page}");
    assert!(!page.contains("data-action=\"plan\""), "{page}");
    assert!(page.contains("--allow-write"), "{page}");
}

#[tokio::test]
async fn the_settings_page_says_where_the_vault_and_root_came_from() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let page = server.page(&cookie, "/settings").await;

    assert!(page.contains("(flag)"), "{page}");
    assert!(page.contains("coding, tmux"), "{page}");
    assert!(page.contains("not allowed"), "{page}");
}

#[tokio::test]
async fn the_doctor_page_lists_findings_or_says_all_is_well() {
    let server = Server::new(false);
    let cookie = server.login().await;
    let page = server.page(&cookie, "/doctor").await;
    assert!(page.contains("Doctor"), "{page}");
    assert!(
        page.contains("SKILL.md") || page.contains("Everything checked out"),
        "{page}"
    );
}

#[tokio::test]
async fn the_history_page_is_empty_before_the_first_run() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let page = server.page(&cookie, "/history").await;
    assert!(page.contains("No run has anything to undo."), "{page}");
}

#[tokio::test]
async fn the_report_is_served_with_its_own_policy_and_its_marker() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let response = server.get(&cookie, "/report").await;

    assert_eq!(response.status(), 200);
    let csp = response.headers()[axum::http::header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap()
        .to_string();
    assert!(csp.contains("script-src 'unsafe-inline'"), "{csp}");
    assert!(csp.contains("default-src 'none'"), "{csp}");
    assert!(
        !csp.contains("connect-src"),
        "the report may not send anything: {csp}"
    );
    let page = super::text(response).await;
    assert!(page.contains(skillmirror_web_marker()), "{page}");
}

fn skillmirror_web_marker() -> &'static str {
    crate::MARKER
}

#[tokio::test]
async fn the_style_sheet_and_the_script_are_served_with_their_types() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let css = server.get(&cookie, "/app.css").await;
    assert_eq!(
        css.headers()[axum::http::header::CONTENT_TYPE],
        "text/css; charset=utf-8"
    );
    let js = server.get(&cookie, "/app.js").await;
    assert_eq!(
        js.headers()[axum::http::header::CONTENT_TYPE],
        "text/javascript; charset=utf-8"
    );
}

#[tokio::test]
async fn a_vault_that_is_not_set_gives_an_error_page_with_the_reason() {
    let world = World::new();
    let server = Server::of(world, false);
    let cookie = server.login().await;
    // The flags point at a vault that does not exist.
    let response = server.get(&cookie, "/").await;
    assert_eq!(response.status(), 500);
    let page = super::text(response).await;
    assert!(page.contains("This page cannot be made"), "{page}");
    assert!(page.contains("doctor"), "{page}");
}

#[tokio::test]
async fn an_unknown_path_is_404_behind_the_guard() {
    let server = Server::new(false);
    let cookie = server.login().await;
    assert_eq!(server.get(&cookie, "/nope").await.status(), 404);
    assert_eq!(server.send(super::visit("/nope")).await.status(), 401);
}
