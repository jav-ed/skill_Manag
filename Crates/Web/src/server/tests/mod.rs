//! Tests of the server, driven in-process: requests go straight into the router.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

mod flow;
mod guard;
mod idle;
mod pages;
mod undo;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, header};
use http_body_util::BodyExt;
use skillmirror_core::config::{Dirs, EnvOverrides, Flags};
use skillmirror_testkit::World;
use tower::ServiceExt;

use super::router;
use super::state::{AppState, Config};

pub(super) const PORT: u16 = 41234;
pub(super) const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// The server of one throwaway world.
pub(super) struct Server {
    pub(super) world: World,
    pub(super) state: Arc<AppState>,
    pub(super) router: Router,
}

impl Server {
    pub(super) fn new(write: bool) -> Self {
        Self::of(World::standard(), write)
    }

    pub(super) fn of(world: World, write: bool) -> Self {
        let config = Config {
            flags: Flags {
                vault: Some(world.vault()),
                root: Some(world.root()),
            },
            env: EnvOverrides::default(),
            dirs: Dirs::under(&world.path().join("home")),
            allow_write: write,
        };
        let state = Arc::new(AppState::new(config, PORT, TOKEN.to_string()));
        let router = router(Arc::clone(&state));
        Self {
            world,
            state,
            router,
        }
    }

    pub(super) async fn send(&self, request: Request<Body>) -> Response<Body> {
        self.router.clone().oneshot(request).await.unwrap()
    }

    /// Trades the launch token for a session, as a browser does, and returns the cookie to send back.
    pub(super) async fn login(&self) -> String {
        let response = self.send(visit(&format!("/?token={TOKEN}"))).await;
        assert_eq!(response.status(), 303, "the token is traded for a session");
        let set = response.headers()[header::SET_COOKIE].to_str().unwrap();
        set.split(';').next().unwrap().to_string()
    }

    pub(super) async fn get(&self, cookie: &str, path: &str) -> Response<Body> {
        self.send(
            request("GET", path)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }

    pub(super) async fn post(
        &self,
        cookie: &str,
        path: &str,
        json: &serde_json::Value,
    ) -> Response<Body> {
        self.send(
            request("POST", path)
                .header(header::COOKIE, cookie)
                .header(header::CONTENT_TYPE, "application/json")
                .header("x-skillmirror", "1")
                .body(Body::from(json.to_string()))
                .unwrap(),
        )
        .await
    }

    pub(super) async fn page(&self, cookie: &str, path: &str) -> String {
        let response = self.get(cookie, path).await;
        assert_eq!(response.status(), 200, "{path}");
        text(response).await
    }

    pub(super) async fn json(
        &self,
        cookie: &str,
        path: &str,
        json: &serde_json::Value,
    ) -> (u16, serde_json::Value) {
        let response = self.post(cookie, path, json).await;
        let status = response.status().as_u16();
        let body = body_bytes(response).await;
        (
            status,
            serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
        )
    }

    /// Waits for the job to end and returns its last view.
    pub(super) async fn finish(&self, cookie: &str, job: &str) -> serde_json::Value {
        for _ in 0..200 {
            let response = self.get(cookie, &format!("/api/job/{job}")).await;
            let body: serde_json::Value =
                serde_json::from_slice(&body_bytes(response).await).unwrap();
            if body["state"] != "running" {
                return body;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        panic!("the job did not end");
    }
}

/// A request with no headers at all.
pub(super) fn bare(method: &str, path: &str) -> axum::http::request::Builder {
    Request::builder().method(method).uri(path)
}

/// A request to our own address.
pub(super) fn request(method: &str, path: &str) -> axum::http::request::Builder {
    bare(method, path).header(header::HOST, format!("127.0.0.1:{PORT}"))
}

/// A plain visit, as when the person opens the link.
pub(super) fn visit(path: &str) -> Request<Body> {
    request("GET", path).body(Body::empty()).unwrap()
}

pub(super) async fn body_bytes(response: Response<Body>) -> Vec<u8> {
    response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec()
}

pub(super) async fn text(response: Response<Body>) -> String {
    String::from_utf8(body_bytes(response).await).unwrap()
}
