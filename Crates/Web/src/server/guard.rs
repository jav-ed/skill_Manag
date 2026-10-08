//! Who may talk to the server. The server listens on the loopback address only, but a web page in
//! the user's own browser can reach that address too, so every request is checked:
//!
//! - the `Host` header must be the address we bind (a page on a rebound name is refused);
//! - an `Origin` or `Sec-Fetch-Site` header from another site is refused;
//! - the first visit carries a one-time token in the link; it is traded for a session cookie
//!   (`HttpOnly`, `SameSite=Strict`) and never works again;
//! - every other request needs that cookie;
//! - a request that changes anything is a POST with a JSON body and our own header, which a form or an
//!   image tag on another site cannot send.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;

use super::state::AppState;

pub(crate) const COOKIE: &str = "skillmirror_session";
/// A header that a cross-site form cannot set; it also forces a preflight for a cross-site `fetch`.
pub(crate) const MARK: &str = "x-skillmirror";

/// The policy of every page of ours: the style sheet and the script come from this server, and nothing
/// else is loaded or sent anywhere.
pub(crate) const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

/// Compares two secrets in time that does not depend on where they differ.
pub(crate) fn same(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    let mut difference = u8::from(left.len() != right.len());
    for (a, b) in left.iter().zip(right) {
        difference |= a ^ b;
    }
    difference == 0
}

/// `bytes` random bytes as lower-case hex. The system has no random source only when it is broken, and
/// then no token is better than a guessable one.
pub(crate) fn random_hex(bytes: usize) -> Result<String, String> {
    let mut buffer = vec![0u8; bytes];
    getrandom::fill(&mut buffer).map_err(|e| format!("no random numbers from the system: {e}"))?;
    let mut hex = String::with_capacity(bytes * 2);
    for byte in &buffer {
        for nibble in [byte >> 4, byte & 0x0f] {
            hex.extend(char::from_digit(u32::from(nibble), 16));
        }
    }
    Ok(hex)
}

fn text(status: StatusCode, message: &str) -> Response {
    let mut response = Response::new(Body::from(format!("{message}\n")));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    harden(&mut response);
    response
}

/// The headers every response carries. A route that needs another policy sets it first.
pub(crate) fn harden(response: &mut Response) {
    let headers = response.headers_mut();
    for (name, value) in [
        (header::CONTENT_SECURITY_POLICY, CSP),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "no-referrer"),
        (header::X_FRAME_OPTIONS, "DENY"),
        (header::CACHE_CONTROL, "no-store"),
        (
            header::HeaderName::from_static("cross-origin-opener-policy"),
            "same-origin",
        ),
        (
            header::HeaderName::from_static("cross-origin-resource-policy"),
            "same-origin",
        ),
    ] {
        headers
            .entry(name)
            .or_insert_with(|| HeaderValue::from_static(value));
    }
}

fn value(headers: &HeaderMap, name: header::HeaderName) -> Option<&str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}

fn host_allowed(headers: &HeaderMap, port: u16) -> bool {
    value(headers, header::HOST).is_some_and(|host| {
        host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
    })
}

fn origin_allowed(origin: &str, port: u16) -> bool {
    origin == format!("http://127.0.0.1:{port}") || origin == format!("http://localhost:{port}")
}

/// The value of `name` in the `Cookie` header.
fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    value(headers, header::COOKIE)?
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(key, found)| (key == name).then_some(found))
}

/// The `token` of the query string of the first-visit link.
fn token_of(query: Option<&str>) -> Option<&str> {
    query?
        .split('&')
        .find_map(|pair| pair.strip_prefix("token="))
}

fn is_read(method: &Method) -> bool {
    method == Method::GET || method == Method::HEAD
}

/// Why a request is not let in, if it is not.
fn refusal(state: &AppState, request: &Request) -> Option<Response> {
    let headers = request.headers();
    if !host_allowed(headers, state.port) {
        return Some(text(StatusCode::FORBIDDEN, "This address is not served."));
    }
    if let Some(origin) = value(headers, header::ORIGIN)
        && !origin_allowed(origin, state.port)
    {
        return Some(text(
            StatusCode::FORBIDDEN,
            "Requests from other sites are refused.",
        ));
    }
    // `none` is a person typing or clicking the link: only a plain read may come that way.
    match value(headers, header::HeaderName::from_static("sec-fetch-site")) {
        None | Some("same-origin") => {}
        Some("none") if is_read(request.method()) => {}
        Some(_) => {
            return Some(text(
                StatusCode::FORBIDDEN,
                "Requests from other sites are refused.",
            ));
        }
    }
    if !is_read(request.method()) && request.method() != Method::POST {
        return Some(text(
            StatusCode::METHOD_NOT_ALLOWED,
            "Only GET and POST are used.",
        ));
    }
    None
}

pub(crate) async fn check(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    state.touch();
    if let Some(refused) = refusal(&state, &request) {
        return refused;
    }
    if request.method() == Method::GET
        && request.uri().path() == "/"
        && let Some(given) = token_of(request.uri().query())
    {
        return trade_token(&state, given);
    }
    let known = cookie(request.headers(), COOKIE).is_some_and(|id| state.is_session(id));
    if !known {
        return text(
            StatusCode::UNAUTHORIZED,
            "Open the link that `skillmirror web` printed in the terminal.",
        );
    }
    if request.method() == Method::POST && !marked_json(request.headers()) {
        return text(
            StatusCode::FORBIDDEN,
            "A change must be sent as JSON with the X-Skillmirror header.",
        );
    }
    let mut response = next.run(request).await;
    harden(&mut response);
    response
}

fn marked_json(headers: &HeaderMap) -> bool {
    headers.get(MARK).is_some_and(|v| v == "1")
        && value(headers, header::CONTENT_TYPE).is_some_and(|t| t.starts_with("application/json"))
}

/// The first visit: the token becomes a session and the link is replaced by a plain one.
fn trade_token(state: &AppState, given: &str) -> Response {
    if !state.spend_token(given) {
        return text(
            StatusCode::FORBIDDEN,
            "This link was used already. Start `skillmirror web` again for a new one.",
        );
    }
    let Ok(session) = random_hex(32) else {
        return text(
            StatusCode::INTERNAL_SERVER_ERROR,
            "No random numbers for a session.",
        );
    };
    state.add_session(session.clone());
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::SEE_OTHER;
    let headers = response.headers_mut();
    headers.insert(header::LOCATION, HeaderValue::from_static("/"));
    if let Ok(cookie) = HeaderValue::from_str(&format!(
        "{COOKIE}={session}; HttpOnly; SameSite=Strict; Path=/"
    )) {
        headers.insert(header::SET_COOKIE, cookie);
    }
    harden(&mut response);
    response
}
