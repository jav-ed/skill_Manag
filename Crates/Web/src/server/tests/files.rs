//! The web interface as the binary serves it: the static files of `Ui/`, and the report.

use axum::http::header;

use super::{Server, text};
use crate::server::files;

const VIEWS: [&str; 7] = [
    "/",
    "/skills",
    "/sync",
    "/push",
    "/history",
    "/doctor",
    "/settings",
];

#[tokio::test]
async fn every_view_address_serves_the_same_shell() {
    let server = Server::new(false);
    let cookie = server.login().await;

    for path in VIEWS {
        let response = server.get(&cookie, path).await;
        assert_eq!(response.status(), 200, "{path}");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/html; charset=utf-8",
            "{path}"
        );
        let page = text(response).await;
        assert!(page.contains("<div id=\"app\">"), "{path}: {page}");
    }
    // A trailing slash is the same address.
    assert_eq!(server.get(&cookie, "/sync/").await.status(), 200);
}

#[tokio::test]
async fn an_address_that_is_not_a_view_or_a_file_is_404() {
    let server = Server::new(false);
    let cookie = server.login().await;
    for path in [
        "/nope",
        "/_astro/missing.js",
        "/sync/other",
        "/index.html.bak",
    ] {
        assert_eq!(server.get(&cookie, path).await.status(), 404, "{path}");
    }
}

#[tokio::test]
async fn no_address_reaches_anything_but_the_table() {
    let server = Server::new(false);
    let cookie = server.login().await;
    for path in [
        "/../Cargo.toml",
        "/%2e%2e/Cargo.toml",
        "/_astro/../../../etc/passwd",
        "/..%2f..%2fetc/passwd",
        "/a\\b",
    ] {
        let response = server.get(&cookie, path).await;
        assert_eq!(response.status(), 404, "{path}");
    }
}

#[tokio::test]
async fn files_come_with_the_right_content_type() {
    let server = Server::new(false);
    let cookie = server.login().await;
    let theme = server.get(&cookie, "/theme.js").await;
    assert_eq!(
        theme.headers()[header::CONTENT_TYPE],
        "text/javascript; charset=utf-8"
    );
    let icon = server.get(&cookie, "/favicon.svg").await;
    assert_eq!(icon.headers()[header::CONTENT_TYPE], "image/svg+xml");
    let page = server.page(&cookie, "/").await;
    let css = page
        .split("href=\"")
        .find_map(|part| part.strip_prefix("/_astro/"))
        .unwrap();
    let css = format!("/_astro/{}", css.split('"').next().unwrap());
    assert_eq!(
        css.rsplit_once('.').map(|(_, ext)| ext),
        Some("css"),
        "{css}"
    );
    let response = server.get(&cookie, &css).await;
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "text/css; charset=utf-8"
    );
}

/// The part of a file name after its last dot.
fn extension(name: &str) -> &str {
    name.rsplit_once('.').map_or("", |(_, ext)| ext)
}

/// Every opening tag of `name` in `html`, up to its `>`.
fn tags<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("<{name}");
    html.match_indices(&open)
        .filter(|(at, _)| {
            html[at + open.len()..]
                .chars()
                .next()
                .is_some_and(|c| c == ' ' || c == '>' || c == '/')
        })
        .map(|(at, _)| &html[at..=at + html[at..].find('>').unwrap()])
        .collect()
}

#[test]
fn no_page_holds_a_script_a_style_or_a_handler_of_its_own() {
    // The policy of the server allows scripts and styles from the server only. A page that carried its own
    // would not run; this test says so before the browser does.
    let pages: Vec<_> = files::all()
        .iter()
        .filter(|(name, _, _)| extension(name) == "html")
        .collect();
    assert_eq!(pages.len(), 7, "one page per view");
    for (name, _, bytes) in pages {
        let html = std::str::from_utf8(bytes).unwrap();
        for script in tags(html, "script") {
            assert!(
                script.contains(" src=\""),
                "{name}: an inline script: {script}"
            );
        }
        assert!(tags(html, "style").is_empty(), "{name}: an inline style");
        assert!(!html.contains(" style=\""), "{name}: a style attribute");
        for handler in [" onclick=", " onload=", " onerror=", "javascript:"] {
            assert!(!html.contains(handler), "{name}: {handler}");
        }
        for tag in tags(html, "script").into_iter().chain(tags(html, "link")) {
            for attribute in [" src=\"", " href=\""] {
                if let Some(rest) = tag.split(attribute).nth(1) {
                    assert!(
                        rest.starts_with('/') && !rest.starts_with("//"),
                        "{name}: a file from elsewhere: {tag}"
                    );
                }
            }
        }
    }
}

#[test]
fn the_built_interface_has_nothing_that_looks_at_the_network_but_the_server() {
    for (name, _, bytes) in files::all() {
        if extension(name) != "js" {
            continue;
        }
        let code = std::str::from_utf8(bytes).unwrap();
        // The server's own pages talk to relative addresses only.
        for outside in [
            "https://",
            "http://",
            "ws://",
            "wss://",
            "importScripts",
            "navigator.sendBeacon",
        ] {
            let hits = code.matches(outside).count();
            let allowed = code.matches("http://www.w3.org").count()
                + code.matches("https://github.com").count();
            assert!(hits <= allowed, "{name} mentions {outside} ({hits})");
        }
    }
}

#[tokio::test]
async fn the_report_is_served_with_its_own_policy_and_its_marker() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let response = server.get(&cookie, "/report").await;

    assert_eq!(response.status(), 200);
    let csp = response.headers()[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap()
        .to_string();
    assert!(csp.contains("script-src 'unsafe-inline'"), "{csp}");
    assert!(csp.contains("default-src 'none'"), "{csp}");
    assert!(
        !csp.contains("connect-src"),
        "the report may not send anything: {csp}"
    );
    let page = text(response).await;
    assert!(page.contains(crate::MARKER), "{page}");
}

#[tokio::test]
async fn a_vault_that_is_not_set_gives_the_report_error_as_text() {
    let server = Server::of(skillmirror_testkit::World::new(), false);
    let cookie = server.login().await;
    let response = server.get(&cookie, "/report").await;
    assert_eq!(response.status(), 500);
    assert!(text(response).await.contains("The report cannot be made"));
}
