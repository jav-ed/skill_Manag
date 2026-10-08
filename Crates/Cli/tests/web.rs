//! `skillmirror web` as a real process: it prints a link once, listens on the loopback address only,
//! and answers a browser's first visit.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use skillmirror_testkit::World;

struct Running {
    child: Child,
    port: u16,
    token: String,
    first_lines: Vec<String>,
}

impl Drop for Running {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}

fn start(world: &World, extra: &[&str]) -> Running {
    let mut command = Command::new(env!("CARGO_BIN_EXE_skillmirror"));
    command
        .env_clear()
        .envs(world.env())
        .arg("--vault")
        .arg(world.vault())
        .arg("--root")
        .arg(world.root())
        .arg("web")
        .args(extra)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap());
    let mut first_lines = Vec::new();
    let link = loop {
        let mut line = String::new();
        assert!(
            lines.read_line(&mut line).unwrap() > 0,
            "the server ended: {first_lines:?}"
        );
        let line = line.trim().to_string();
        first_lines.push(line.clone());
        if let Some(link) = line.strip_prefix("Open: ") {
            break link.to_string();
        }
    };
    let rest = link
        .strip_prefix("http://127.0.0.1:")
        .expect("a loopback link");
    let (port, query) = rest.split_once("/?token=").unwrap();
    Running {
        child,
        port: port.parse().unwrap(),
        token: query.to_string(),
        first_lines,
    }
}

/// One HTTP/1.1 request on a fresh connection; the whole answer as text.
fn http(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut answer = Vec::new();
    // The server keeps the connection open; read until the declared body is complete.
    let mut buffer = [0u8; 4096];
    loop {
        let n = match stream.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        answer.extend_from_slice(&buffer[..n]);
        let text = String::from_utf8_lossy(&answer);
        if let Some((head, body)) = text.split_once("\r\n\r\n") {
            let length = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length: ")
                        .map(str::to_string)
                })
                .and_then(|v| v.trim().parse::<usize>().ok());
            if length.is_none_or(|len| body.len() >= len) {
                break;
            }
        }
    }
    String::from_utf8_lossy(&answer).into_owned()
}

fn get(port: u16, path: &str, extra: &str) -> String {
    http(
        port,
        &format!(
            "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n{extra}\r\n"
        ),
    )
}

#[test]
fn the_server_prints_one_link_listens_on_loopback_only_and_trades_it_for_a_session() {
    let world = World::standard();
    let server = start(&world, &["--port", "0"]);

    assert!(
        server.first_lines[0].contains("read-only"),
        "{:?}",
        server.first_lines
    );
    assert_eq!(server.token.len(), 64);
    // The socket is bound to 127.0.0.1 and to nothing else.
    let table = std::fs::read_to_string("/proc/net/tcp").unwrap();
    let wanted = format!(":{:04X}", server.port);
    let listening: Vec<&str> = table
        .lines()
        .skip(1)
        .filter(|l| {
            l.split_whitespace()
                .nth(1)
                .is_some_and(|a| a.ends_with(&wanted))
        })
        .filter(|l| l.split_whitespace().nth(3) == Some("0A"))
        .collect();
    assert_eq!(listening.len(), 1, "one listening socket: {listening:?}");
    assert!(
        listening[0]
            .split_whitespace()
            .nth(1)
            .unwrap()
            .starts_with("0100007F:"),
        "bound to 127.0.0.1, not to every address: {listening:?}"
    );

    let first = get(server.port, &format!("/?token={}", server.token), "");
    assert!(first.starts_with("HTTP/1.1 303"), "{first}");
    let cookie = first
        .lines()
        .find_map(|l| l.strip_prefix("set-cookie: "))
        .expect("a cookie")
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let page = get(server.port, "/", &format!("Cookie: {cookie}\r\n"));
    assert!(page.starts_with("HTTP/1.1 200"), "{page}");
    assert!(page.contains("Overview"), "{page}");
    let again = get(server.port, &format!("/?token={}", server.token), "");
    assert!(
        again.starts_with("HTTP/1.1 403"),
        "the link works once: {again}"
    );
    let stranger = get(server.port, "/", "");
    assert!(stranger.starts_with("HTTP/1.1 401"), "{stranger}");
}

#[test]
fn a_wrong_host_header_gets_nothing_even_with_the_session() {
    let world = World::standard();
    let server = start(&world, &[]);
    let first = get(server.port, &format!("/?token={}", server.token), "");
    let cookie = first
        .lines()
        .find_map(|l| l.strip_prefix("set-cookie: "))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();

    let answer = http(
        server.port,
        &format!(
            "GET / HTTP/1.1\r\nHost: evil.example\r\nCookie: {cookie}\r\nConnection: close\r\n\r\n"
        ),
    );

    assert!(answer.starts_with("HTTP/1.1 403"), "{answer}");
    assert!(!answer.contains("Overview"));
}

#[test]
fn allow_write_is_announced_and_ctrl_c_ends_the_server_cleanly() {
    let world = World::standard();
    let mut server = start(&world, &["--allow-write"]);
    assert!(
        server.first_lines[0].contains("changes allowed"),
        "{:?}",
        server.first_lines
    );

    let pid = server.child.id().to_string();
    let killed = Command::new("kill").args(["-INT", &pid]).status().unwrap();
    assert!(killed.success());

    let status = server.child.wait().unwrap();
    assert_eq!(status.code(), Some(0), "Ctrl-C is a normal end");
}

#[test]
fn it_refuses_to_start_without_a_vault_and_prints_no_link() {
    let world = World::new();
    let output = Command::new(env!("CARGO_BIN_EXE_skillmirror"))
        .env_clear()
        .envs(world.env())
        .arg("web")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("token="));
    assert!(String::from_utf8_lossy(&output.stderr).contains("vault"));
}

#[test]
fn a_taken_port_is_an_error_with_the_port_in_it() {
    let world = World::standard();
    let blocker = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = blocker.local_addr().unwrap().port().to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_skillmirror"))
        .env_clear()
        .envs(world.env())
        .arg("--vault")
        .arg(world.vault())
        .arg("--root")
        .arg(world.root())
        .args(["web", "--port", &port])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&output.stderr).contains(&port));
}

#[test]
fn the_idle_timeout_ends_a_forgotten_server_and_a_bad_value_is_refused() {
    let world = World::standard();
    // 0.03 minutes: the server must be there at first and gone about two seconds later.
    let mut server = start(&world, &["--idle-timeout", "0.03"]);
    assert!(
        server.child.try_wait().unwrap().is_none(),
        "still serving at first"
    );

    let mut status = None;
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(100));
        if let Some(done) = server.child.try_wait().unwrap() {
            status = Some(done);
            break;
        }
    }

    assert_eq!(status.expect("it ended by itself").code(), Some(0));
    let bad = Command::new(env!("CARGO_BIN_EXE_skillmirror"))
        .env_clear()
        .envs(world.env())
        .args(["web", "--idle-timeout", "-3"])
        .output()
        .unwrap();
    assert_eq!(
        bad.status.code(),
        Some(2),
        "a command line that cannot be carried out"
    );
}
