
default:
    @just --list

[doc('Delete all Scratch contents and recreate empty Agent_Tasks, Audit, Design, and Screenshots folders.')]
scratch-clean:
    bash Code/Development/Scratch/clean.sh

# Format every crate in place.
fmt:
    cargo fmt --all

# Fail when any crate is not formatted (used by check and CI).
fmt-check:
    cargo fmt --all -- --check

# Lint the whole workspace, all targets; every warning is an error.
clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

# Run all tests with nextest, then the doc tests (nextest does not run those).
test:
    cargo nextest run --workspace --locked
    cargo test --doc --workspace --locked

# Fail when a Rust file has more than 300 code lines (tokei, see loc_gate.sh).
loc-gate:
    #!/usr/bin/env bash
    set -euo pipefail
    # Tracked plus not-yet-committed files; anything matched by .gitignore is skipped.
    files=()
    while IFS= read -r -d '' file; do
        if [ -f "$file" ]; then
            files+=("$file")
        fi
    done < <(git ls-files -z --cached --others --exclude-standard -- '*.rs')
    if [ "${#files[@]}" -eq 0 ]; then
        echo "loc-gate: git lists no .rs files, nothing to check" >&2
        exit 2
    fi
    bash Code/Development/Scripts/loc_gate.sh "${files[@]}"

# Fail when skillmirror-core (normal dependencies, all features) pulls in tokio: core stays synchronous.
check-deps:
    #!/usr/bin/env bash
    set -uo pipefail
    # The package must resolve first, otherwise a typo in the name would look like a pass.
    if ! cargo tree --locked -p skillmirror-core --depth 0 >/dev/null; then
        echo "check-deps: cannot read the dependency tree of skillmirror-core (package missing or Cargo.lock out of date)" >&2
        exit 1
    fi
    err=$(mktemp)
    trap 'rm -f "$err"' EXIT
    # A tokio in the tree prints an inverted tree whose first line is "tokio v...". No tokio in the
    # tree makes cargo fail ("did not match any packages") or print nothing: both are a pass.
    if tree=$(cargo tree --locked -p skillmirror-core -e normal --all-features -i tokio 2>"$err"); then
        status=0
    else
        status=$?
    fi
    if printf '%s\n' "$tree" | grep -q '^tokio v'; then
        echo "check-deps: skillmirror-core depends on tokio, but core must stay synchronous. Reverse path:" >&2
        printf '%s\n' "$tree" >&2
        exit 1
    fi
    if [ "$status" -ne 0 ] && ! grep -q 'did not match any packages' "$err"; then
        echo "check-deps: 'cargo tree -i tokio' failed for an unexpected reason:" >&2
        cat "$err" >&2
        exit 1
    fi
    echo "check-deps: ok, skillmirror-core does not depend on tokio"

# Supply chain: advisories, licences, bans and sources (needs cargo-deny, see deny.toml).
deny:
    cargo deny --locked check

# Install the skillmirror binary from this checkout into ~/.cargo/bin.
install:
    cargo install --path Crates/Cli --locked

# The local gate before a commit: format, lint, tests, file size and the core-without-tokio rule.
check: fmt-check clippy test loc-gate check-deps

# Compile against the declared minimum Rust version (run `rustup toolchain install 1.88` once).
msrv:
    rustup run 1.88 cargo check --locked --workspace
