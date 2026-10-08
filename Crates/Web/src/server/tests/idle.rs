//! A forgotten server ends by itself.

use std::time::Duration;

use super::Server;
use crate::server::idle_over;

#[tokio::test]
async fn a_server_nobody_visits_ends_after_the_idle_time() {
    let server = Server::new(false);
    let started = std::time::Instant::now();

    idle_over(
        std::sync::Arc::clone(&server.state),
        Some(Duration::from_millis(120)),
    )
    .await;

    assert!(started.elapsed() >= Duration::from_millis(120));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[tokio::test]
async fn a_request_pushes_the_end_back() {
    let server = Server::new(false);
    let state = std::sync::Arc::clone(&server.state);
    let waiting = tokio::spawn(idle_over(state, Some(Duration::from_millis(400))));
    for _ in 0..4 {
        tokio::time::sleep(Duration::from_millis(150)).await;
        server.send(super::visit("/")).await;
    }
    assert!(
        !waiting.is_finished(),
        "the visits kept it alive for 600 ms"
    );
    waiting.await.unwrap();
}

#[tokio::test]
async fn without_a_limit_it_never_ends() {
    let server = Server::new(false);
    let waiting = tokio::spawn(idle_over(std::sync::Arc::clone(&server.state), None));
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!waiting.is_finished());
    waiting.abort();
}
