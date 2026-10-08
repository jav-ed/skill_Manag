//! Who gets in: the host, the origin, the one-time token, the session and the shape of a change.

use axum::body::Body;
use axum::http::header;

use super::{PORT, Server, TOKEN, bare, request, text, visit};

#[tokio::test]
async fn a_visit_without_a_session_is_turned_away_with_the_way_in() {
    let server = Server::new(false);

    let response = server.send(visit("/")).await;

    assert_eq!(response.status(), 401);
    assert!(text(response).await.contains("skillmirror web"));
}

#[tokio::test]
async fn the_token_is_traded_once_for_a_session_cookie_that_cannot_be_read_by_scripts() {
    let server = Server::new(false);

    let first = server.send(visit(&format!("/?token={TOKEN}"))).await;

    assert_eq!(first.status(), 303);
    assert_eq!(first.headers()[header::LOCATION], "/");
    let cookie = first.headers()[header::SET_COOKIE]
        .to_str()
        .unwrap()
        .to_string();
    assert!(cookie.contains("HttpOnly"), "{cookie}");
    assert!(cookie.contains("SameSite=Strict"), "{cookie}");
    assert!(cookie.contains("Path=/"), "{cookie}");
    let again = server.send(visit(&format!("/?token={TOKEN}"))).await;
    assert_eq!(again.status(), 403, "a used link works no more");
    assert!(again.headers().get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn a_wrong_token_gives_no_session() {
    let server = Server::new(false);

    let response = server
        .send(visit(&format!("/?token={}", "0".repeat(64))))
        .await;

    assert_eq!(response.status(), 403);
    assert!(response.headers().get(header::SET_COOKIE).is_none());
    assert_eq!(server.send(visit("/")).await.status(), 401);
}

#[tokio::test]
async fn a_made_up_cookie_is_no_session() {
    let server = Server::new(false);
    let response = server.get("skillmirror_session=abcdef", "/").await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn the_session_opens_the_pages() {
    let server = Server::new(false);
    let cookie = server.login().await;
    assert_eq!(server.get(&cookie, "/").await.status(), 200);
}

#[tokio::test]
async fn a_host_that_is_not_ours_is_refused_even_with_a_session() {
    let server = Server::new(false);
    let cookie = server.login().await;

    for host in [
        Some("attacker.example:41234"),
        Some("attacker.example"),
        Some("127.0.0.1"),
        Some("127.0.0.1:1"),
        Some("127.0.0.1.attacker.example:41234"),
        Some(""),
        None,
    ] {
        let mut builder = bare("GET", "/").header(header::COOKIE, &cookie);
        if let Some(host) = host {
            builder = builder.header(header::HOST, host);
        }

        let response = server.send(builder.body(Body::empty()).unwrap()).await;

        assert_eq!(response.status(), 403, "{host:?} must not reach the pages");
    }
}

#[tokio::test]
async fn localhost_is_ours_too() {
    let server = Server::new(false);
    let cookie = server.login().await;
    let response = server
        .send(
            bare("GET", "/")
                .header(header::HOST, format!("localhost:{PORT}"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn a_request_from_another_origin_is_refused() {
    let server = Server::new(false);
    let cookie = server.login().await;

    let foreign = request("GET", "/")
        .header(header::COOKIE, &cookie)
        .header(header::ORIGIN, "https://evil.example")
        .body(Body::empty())
        .unwrap();
    assert_eq!(server.send(foreign).await.status(), 403);
    let ours = request("GET", "/")
        .header(header::COOKIE, &cookie)
        .header(header::ORIGIN, format!("http://127.0.0.1:{PORT}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(server.send(ours).await.status(), 200);
}

#[tokio::test]
async fn the_fetch_metadata_of_another_site_is_refused() {
    let server = Server::new(false);
    let cookie = server.login().await;

    for site in ["cross-site", "same-site"] {
        let request = request("GET", "/")
            .header(header::COOKIE, &cookie)
            .header("sec-fetch-site", site)
            .body(Body::empty())
            .unwrap();
        assert_eq!(server.send(request).await.status(), 403, "{site}");
    }
    for site in ["same-origin", "none"] {
        let request = request("GET", "/")
            .header(header::COOKIE, &cookie)
            .header("sec-fetch-site", site)
            .body(Body::empty())
            .unwrap();
        assert_eq!(server.send(request).await.status(), 200, "{site}");
    }
}

#[tokio::test]
async fn typing_the_address_is_fine_for_a_read_but_never_for_a_change() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let post = request("POST", "/api/rescan")
        .header(header::COOKIE, &cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-skillmirror", "1")
        .header("sec-fetch-site", "none")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(server.send(post).await.status(), 403);
}

#[tokio::test]
async fn a_change_needs_json_and_our_header() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let build = |content: Option<&str>, mark: Option<&str>| {
        let mut builder = request("POST", "/api/rescan").header(header::COOKIE, &cookie);
        if let Some(content) = content {
            builder = builder.header(header::CONTENT_TYPE, content);
        }
        if let Some(mark) = mark {
            builder = builder.header("x-skillmirror", mark);
        }
        builder.body(Body::from("{}")).unwrap()
    };

    assert_eq!(
        server
            .send(build(Some("application/json"), None))
            .await
            .status(),
        403
    );
    assert_eq!(
        server
            .send(build(Some("application/json"), Some("0")))
            .await
            .status(),
        403
    );
    assert_eq!(
        server
            .send(build(Some("application/x-www-form-urlencoded"), Some("1")))
            .await
            .status(),
        403
    );
    assert_eq!(server.send(build(None, Some("1"))).await.status(), 403);
    assert_eq!(
        server
            .send(build(Some("application/json"), Some("1")))
            .await
            .status(),
        200
    );
}

#[tokio::test]
async fn only_get_and_post_exist() {
    let server = Server::new(true);
    let cookie = server.login().await;
    for method in ["PUT", "DELETE", "PATCH", "OPTIONS"] {
        let request = request(method, "/")
            .header(header::COOKIE, &cookie)
            .body(Body::empty())
            .unwrap();
        assert_eq!(server.send(request).await.status(), 405, "{method}");
    }
}

#[tokio::test]
async fn a_change_without_a_session_is_refused_before_anything_else() {
    let server = Server::new(true);
    let request = request("POST", "/api/rescan")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-skillmirror", "1")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(server.send(request).await.status(), 401);
}

#[tokio::test]
async fn an_oversize_body_is_refused() {
    let server = Server::new(true);
    let cookie = server.login().await;
    let big = format!(
        "{{\"kind\":\"sync\",\"skills\":[\"{}\"]}}",
        "a".repeat(100_000)
    );
    let request = request("POST", "/api/plan")
        .header(header::COOKIE, &cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-skillmirror", "1")
        .body(Body::from(big))
        .unwrap();
    assert_eq!(server.send(request).await.status(), 413);
}

#[tokio::test]
async fn every_answer_carries_the_security_headers_and_no_cors() {
    let server = Server::new(false);
    let cookie = server.login().await;

    for path in [
        "/",
        "/theme.js",
        "/favicon.svg",
        "/api/settings",
        "/settings",
        "/nonexistent",
    ] {
        let response = server.get(&cookie, path).await;
        let headers = response.headers();
        let csp = headers[header::CONTENT_SECURITY_POLICY].to_str().unwrap();
        assert!(
            csp.contains("default-src 'none'") && csp.contains("script-src 'self'"),
            "{path}: {csp}"
        );
        assert!(!csp.contains("unsafe-inline"), "{path}: {csp}");
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff", "{path}");
        assert_eq!(headers[header::X_FRAME_OPTIONS], "DENY", "{path}");
        assert_eq!(headers[header::REFERRER_POLICY], "no-referrer", "{path}");
        assert_eq!(headers[header::CACHE_CONTROL], "no-store", "{path}");
        assert!(
            headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none(),
            "{path}"
        );
    }
}

#[tokio::test]
async fn refusals_carry_the_headers_too() {
    let server = Server::new(false);
    let response = server.send(visit("/")).await;
    assert_eq!(response.status(), 401);
    assert_eq!(
        response.headers()[header::X_CONTENT_TYPE_OPTIONS],
        "nosniff"
    );
}

#[test]
fn the_secret_comparison_does_not_confuse_prefixes_or_lengths() {
    use super::super::guard::same;
    assert!(same("abc", "abc"));
    assert!(!same("abc", "abd"));
    assert!(!same("abc", "abcd"));
    assert!(!same("", "a"));
    assert!(same("", ""));
}

#[test]
fn random_words_are_the_asked_length_and_differ() {
    use super::super::guard::random_hex;
    let (a, b) = (random_hex(32).unwrap(), random_hex(32).unwrap());
    assert_eq!(a.len(), 64);
    assert_ne!(a, b);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
}
