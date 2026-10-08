# Rust stack: Engine

Decision: build the engine on `ignore` (parallel walk, 12 threads) plus a JSON project registry, `gix-index` with `gix-discover` for the git index, `ignore` again for untracked-file detection, plain `std` for comparison (size, then bytes, no hash), `rustix` `RENAME_EXCHANGE` plus `tempfile` and `fs-err` for the atomic apply, `std::os::unix::fs::symlink` for the agent-directory links, `similar` for diffs, `serde-saphyr` for reading YAML (config and `SKILL.md` frontmatter), a small line-based editor with mandatory re-parse verification for writing `config.yaml` (with `yaml-edit` as the maybe), `std::sync::mpsc` plus `rayon` for events and parallelism, `serde_json` for the registry, `thiserror` for errors, and `etcetera` for XDG paths. No tokio, no git2, no jwalk, no hash crate for equality. Evidence: every number below was measured on this machine on 2026-10-07 from prototypes in a scratch workspace, with the load caveat stated in the next section. Headline numbers (warm cache, `1_Code`, 114,425 directories after pruning, interleaved runs, median of 15): the Go walker takes 1,922 ms, `ignore` parallel with 12 threads 417 ms (4.6 times faster), a hand-rolled rayon plus `getdents64` walker 227 ms (8.5 times faster); 395 `git ls-files` spawns cost 518 ms versus 0.10 ms in-process with `gix-index`; a no-op plan over 395 targets with byte comparison costs 38 ms on 8 threads, so hashes are not needed for equality; staged apply with `RENAME_EXCHANGE` over all 395 targets takes 274 ms on 8 threads. Cold-cache numbers were not measured (no root to drop caches).

Related files: [Requirements](rewrite_Requirements.md): the questions this file answers. [CLI and quality stack](cli_Stack.md): `thiserror`, config paths and test tooling overlap. [Prior art](prior_Art.md): the lock file shape that the hash question serves. [Web view](web_View.md): consumes the event stream defined here.

## Benchmarks

### Machine and method

- Hardware and system: Intel Core i5-1235U (2 performance plus 8 efficiency cores, 12 threads), 31 GB RAM, ext4 on NVMe with `noatime` for `/` and `/home` (two separate filesystems), kernel 6.1.149, CPU governor `powersave`, rustc 1.98.1, Go 1.27.0, glibc 2.42, git 2.55.0.
- Load caveat: five other research agents compiled and benchmarked at the same time. `loadavg` was 8 to 33 on 12 threads (stated per run below). Absolute wall times are therefore inflated and noisy (a single GNU `find` run varied 4.4 to 6.6 s). Mitigation: all walker variants ran interleaved round-robin in one process (round 1 warm-up, then 15 rounds), the Go walker ran in the same rounds as a process, and I report median, minimum and CPU time (`getrusage`, user plus sys). Read ratios, not absolutes.
- The tree grew while I worked because other agents cloned repositories into `Repos/` inside the scan root: 111,536 directories at 16:10, 114,425 at 17:12; result set 65 skills directories at 16:10, 71 at 17:12. Every approach returned the identical set in every verification pass (sorted paths compared against GNU `find` output with `LC_ALL=C sort`, result `same` for all walkers, `DIFF` only for the gitignore-aware variants, see below).
- Scan semantics reproduced from `internal/walker.go`: prune the 16 noise names, never follow symlinks, find a directory named `skills` whose parent is `.agents`, never descend below it. Files are never read, only `d_type` is used (no stat).
- Scan root: `/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code` (read only). At 16:10: 864,824 entries (752,470 files, 111,536 directories, 817 symlinks), 1,687 noise directories pruned.
- Cold numbers: not measured. Reasoning only: a cold walk is bound by random directory-block and inode reads (about 114k directories). The Go baseline of 13 s cold is serial latency. Parallel readers raise queue depth on the NVMe, so the walk should scale much better cold than warm, and `ignore` caps its default at 12 threads (`Repos/ripgrep/crates/ignore/src/walk.rs:1537-1539`) while `.threads(n)` accepts more. Hypothesis, not evidence: 32 to 64 threads help cold runs. To measure: `echo 3 | sudo tee /proc/sys/vm/drop_caches` then run one `walkbench` pass per thread count.

### Walk: single thread (Go versus Rust, matrix C, loadavg 8 to 11, median of 15 rounds)

| Approach | wall median ms | wall min ms | CPU median ms |
|---|---|---|---|
| Go `WalkDir` extract of `walker.go` (process, includes start) | 1,922 | 1,469 | n/a |
| `std::fs::read_dir` recursive, `file_type()` (d_type) | 962 | 686 | 943 |
| `walkdir` 2.5.0 with `filter_entry` and `skip_current_dir` | 1,230 | 795 | 1,210 |
| `ignore::Walk` (filters off, `filter_entry`) | 1,300 | 1,019 | 1,277 |
| rustix `RawDir` (`getdents64`, reused 32 KiB buffer) with `openat` from the parent fd, DFS | 847 | 544 | 829 |

Reference points: the earlier quieter run (loadavg about 8, 9 rounds) gave std 1,157, walkdir 1,174, ignore 1,374, rustix DFS 780. GNU `find` with the same pruning took 4.4 to 6.6 s in this session (cause not investigated). The `find` of the interactive shell is a function backed by `bfs` (its errors read `bfs:`) and took 0.46 to 0.65 s in 5 runs; I did not re-measure it inside the interleaved harness, so treat it as indicative only. The requirements doc says "plain find 1.6 s"; I could not reproduce that number with GNU find and suggest naming the binary there.

### Walk: threads (same run, wall median / min ms)

| Threads | `ignore` parallel | jwalk 0.9.0 | rayon + `getdents64` (hand-rolled) | std lister in same rayon scheme |
|---|---|---|---|---|
| 1 | 1,639 / 993 | not run | 1,055 / 804 | not run |
| 2 | 1,080 / 598 | 1,297 / 1,041 | 575 / 363 | not run |
| 4 | 689 / 463 | 788 / 612 | 367 / 273 | not run |
| 8 | 450 / 339 | 633 / 524 | 238 / 182 | not run |
| 12 | 417 / 323 | 639 / 553 | 227 / 178 | 268 / 242 |

- Scaling: `ignore` reaches 3.9 times its own 1-thread time at 12 threads, the hand-rolled walker 4.6 times; both flatten after 8 threads (the CPU has 4 fast hardware threads and 8 slow ones, and readdir on one directory takes a shared lock in the kernel). CPU time rises with threads (`ignore` 1,612 ms at 1 thread, 3,604 ms at 12), so threads buy wall time with extra work.
- A crossbeam-channel worker pool (one job per directory, 12 threads) gave the same 267 ms as rayon with the raw lister; rayon is the shorter code. Fixed-depth split with fd-relative DFS below depth 5 (8 threads) gave 304 ms: no gain over per-directory tasks.
- Against the Go walker in the same run (medians): `ignore` 12 threads 4.6 times faster, rayon plus `getdents64` 8.5 times faster. Matrix B (loadavg 17 falling to 10, 13 rounds) agrees: Go 1,670, `ignore` 12 threads 376, rayon raw 12 threads 229, jwalk 12 threads 610.
- The gain from `getdents64` over `std` is about 1.2 times in parallel (227 versus 268 ms) and 1.1 to 1.3 times sequentially (847 versus 962 median, 544 versus 686 min). The rest of the gap between `ignore` and the hand-rolled walker is its per-entry work (path building, `DirEntry` wrapping, gitignore plumbing even when disabled).

### Walk: what the gitignore matcher does

`ignore` with `git_ignore(true)` (hidden files still visible) returned 53 directories instead of 71 and ran in 134 ms (12 threads) versus 417 ms. It skips the 18 `.agents/skills` that live in gitignored trees (mostly cloned repositories under the gitignored `Repos/` folder; one Woodpecker backup folder verified with `git check-ignore`). That is a policy change, not just a speed-up, so the walker keeps standard filters off and expresses policy through `exclude_dirs` and `exclude_paths`. Of the 66 skills directories present at 16:44, `git check-ignore` flagged 1 as ignored and 5 sit outside any git repository, so gitignore state cannot define "a project".

### Registry validation (run against the real 65 projects)

| Operation | Cost |
|---|---|
| JSON registry for 65 projects and 580 skill entries: size | 27.9 KB pretty, 20.0 KB compact |
| `serde_json` read / write | 211 us / 103 us |
| `postcard` read / write (15.6 KB) | 105 us / 25 us |
| Validate: `stat` every registered `.agents/skills`, compare mtime | 0.25 ms |
| Validate: `read_dir` every skills directory | 1.6 ms |
| Snapshot idea: `stat` all 114,425 directories (1 / 2 / 4 / 8 / 12 threads) | 157 / 91 / 94 / 56 / 51 ms median |

A registry removes the walk on repeat runs (about 0.4 s to 2 ms with `ignore`). What it cannot see is a new project created elsewhere; only a walk finds it. The directory-mtime snapshot would detect that with a 51 ms stat pass instead of a 227 ms walk, but needs about 11 MB of stored paths (estimate, not built) for a 4.5 times gain: not worth it. Recommended shape: registry as the fast path, explicit `--rescan`, a time-to-live (for example 24 hours) and a background rescan that streams `ProjectFound` events while the cached projects are already shown. Validation by `stat` of the skills directory mtime catches added or removed skills, not edits inside a skill (not needed for the registry).

### Git: index versus subprocess (real vault, 24 skills, 337 tracked files, load 6 to 31)

| Approach | Time |
|---|---|
| 395 spawns of `git -C <skill> ls-files --cached .` (Go behaviour) | 518 ms median at loadavg 6 (1.3 ms each); 2,076 ms at loadavg 31 |
| 24 spawns, one per vault skill (cache per skill) | 41 ms |
| 1 spawn, `git ls-files --cached -z` for the whole vault | 1.9 ms (3.5 ms under load) |
| `gix-discover` plus `gix-index::File::at` plus 24 prefix lists, in process | 0.10 ms |
| `gix` high-level (`open` plus `index()` plus 24 lists) | 0.76 ms |
| `git2` (open, index, 24 prefix filters) | 1.82 ms |
| Untracked, `gix` with `status` and `dirwalk` features (24 pathspec walks) | 15.8 ms |
| Untracked, `git2` `statuses` (24 pathspecs) | 16.7 ms |
| Untracked, `ignore` walk of each skill plus index set difference | 22 to 28 ms |
| Untracked, 1 spawn `git ls-files --others --exclude-standard` | 8 to 10 ms |

Correctness checks: `gix-index` prefix listing equals `git ls-files --cached -s` (symlinks 120000 dropped) for all 24 skills on the real vault, and for a scratch clone with a staged rename, a staged addition, a deleted tracked file and a symlink (0 mismatching skills). The `ignore`-based untracked detection equals `git ls-files --others --exclude-standard` exactly on the clone (5 untracked: a nested directory, a hidden file, an untracked `.gitignore`, an untracked symlink; files ignored by `.git/info/exclude` or a per-skill `.gitignore` were correctly dropped) and on the real vault (0 equals 0). The real vault holds a 138 MB `x-post/code/.venv` (968 files, ignored by its own `.gitignore` containing `*`), which any naive walk would traverse.

Compile cost and size (fresh target dir each, `lto = "thin"`, `debug = false`; CPU seconds are the load-robust figure, wall was inflated by load):

| Variant | crates (excl. root) | compile user CPU | stripped binary | unstripped |
|---|---|---|---|---|
| empty `fn main` (reference) | 0 | n/a | 350 KB | 4.5 MB (`rustc -O`) |
| `gix-index` + `gix-discover` + `gix-hash` (sha1) | 83 | 136 s | 946 KB | 1.21 MB |
| `git2` default-features off | 13 plus vendored libgit2 C | 240 s (real 185 s, C compile) | 1,387 KB | 1.57 MB |
| `gix` features `index`, `sha1` | 128 | 260 s | 1,853 KB | 2.50 MB |
| `gix` features `index`, `sha1`, `status`, `dirwalk` | 161 | 346 s | 2,509 KB | 3.43 MB |

### Compare, hash, plan, apply (synthetic layout: 60 projects, 395 targets, 337 files, from the real vault; written only under `~/.cache/skillres_fs`, loadavg 20 to 30)

| Operation over all 395 targets | median ms | min ms |
|---|---|---|
| Plan no-op, size only, sequential / 8 threads | 67.7 / 44.0 | 40.6 / 27.3 |
| Plan no-op, read dest and compare bytes, sequential / 8 threads | 126.6 / 38.1 | 89.0 / 30.3 |
| Plan no-op, blake3 of dest and source each time, sequential / 8 threads | 238.3 / 90.0 | 181.4 / 72.6 |
| Apply Go style (remove tree, copy), sequential / 8 threads | 619 / 197 | 543 / 181 |
| Apply staged next to target plus `RENAME_EXCHANGE`, sequential | 1,285 | 781 |
| Apply staged plus exchange, 4 / 8 threads | 384 / 274 | 289 / 243 |
| Incremental: byte plan (8 threads) plus apply of the 17 targets that changed | 93 | 64 |
| Write 395 targets with `std::fs::copy` / `fs::write` plus chmod (sequential) | 161 / 153 | 155 / 130 |

Hash micro benchmark (single thread, AVX2 and SHA-NI present, loadavg high so absolute GB/s are low): 50 KB file: copy plus compare 12 us, `xxh3_64` (xxhash-rust) 6.7 us, `twox-hash` XXH3 8.0 us, SHA-256 64 us (0.8 GB/s), blake3 104 us (0.49 GB/s; typically faster, the machine was loaded). The whole vault is 2.0 MB over 338 tracked files, the largest 50,015 bytes, so even SHA-256 hashes everything in about 2.5 ms.

Probes (`fsbench/probe`, `linkprobe`): `std::fs::copy` uses `copy_file_range` (strace); mode 0777, 0751, 0600 and 0444 come through unchanged under umask 022 (a setuid bit is dropped: 4755 becomes 755); `reflink_copy::reflink` fails with `EOPNOTSUPP` on this ext4; `renameat2(RENAME_EXCHANGE)` swaps two directories on ext4 and tmpfs; `rename` of a directory onto a non-empty directory fails with `ENOTEMPTY`; `rename` across devices fails with `EXDEV` (os error 18).

### How to reproduce

The sources are in the session scratchpad `.../scratchpad/engine/` (not in the repo, by instruction). Each folder is a Cargo project built with `CARGO_TARGET_DIR` inside it and `--release`:

- `walkbench`: all walkers behind one harness: `walkbench <root> <rounds> name:threads ...` (names: `std_seq`, `walkdir_seq`, `ignore_seq`, `ignore_par`, `jwalk`, `pool_raw`, `rayon_std`, `rayon_raw`, `raw_dfs_seq`, `raw_dfs_par5`, `gowalk`, `gnufind`, `*_gitignore`). `EXPECT=file` compares against a `LC_ALL=C sort`ed GNU `find` result. `src/bin/dirstat.rs` is the stat-all-directories test. `gowalk/` is `walker.go` reduced to its walk. `run_matrix_*.sh` are the invocations behind the tables.
- `gitbench`: `gix_idx` (with `src/bin/{untracked,tracked_eq,spawn_cmp}.rs`), `gix_min`, `gix_status`, `git2_min`, `build_all.sh`.
- `fsbench`: `apply`, `probe`, `linkprobe`, `hashbench`. `yamlbench`: `main`, `fuzz`, `toml_t`, `line_edit.rs`. `miscbench`: `front`, `chan`, `reg`, `dirs`. `diffbench`. `depcount` (dependency counts, licence census).

The core of the best walker is small (rayon scope, one task per directory, `RawDir` with a thread-local buffer):

```rust
fn list_raw(dir: &Path, subs: &mut Vec<PathBuf>, found: &mut Vec<PathBuf>) {
    let Ok(fd) = openat(CWD, dir, DIR_FLAGS, Mode::empty()) else { return };
    let dn = dir.file_name().unwrap_or_default();
    BUF.with(|b| {
        let mut rd = RawDir::new(&fd, &mut b.borrow_mut()[..]);
        while let Some(Ok(e)) = rd.next() {
            if e.file_type() != FileType::Directory { continue; }
            let name = OsStr::from_bytes(e.file_name().to_bytes());
            if name == "." || name == ".." || is_noise(name) { continue; }
            if is_skills_dir(dn, name) { found.push(dir.join(name)); continue; }
            subs.push(dir.join(name));
        }
    });
}
fn spawn_dir<'a>(s: &rayon::Scope<'a>, dir: PathBuf, all: &'a Mutex<Vec<PathBuf>>) {
    s.spawn(move |s| {
        let (mut subs, mut found) = (Vec::new(), Vec::new());
        list_raw(&dir, &mut subs, &mut found);
        if !found.is_empty() { all.lock().unwrap().extend(found); }
        for d in subs { spawn_dir(s, d, all); }
    });
}
```

## Options considered

### Directory walking

- `ignore` 0.4.33 (released 2026-08-04, 185M downloads, Unlicense OR MIT, BurntSushi, source at `Repos/ripgrep/crates/ignore/`; the ripgrep clone was made by another agent). Build parallel with `WalkBuilder::build_parallel` (`walk.rs:701`), prune in `filter_entry` (`walk.rs:1046`), return `WalkState::Skip` to stop descent after a hit (enum at `walk.rs:1325-1336`: "don't descend into it"). `standard_filters(false)` and `hidden(false)` are required, otherwise `.agents` is skipped as hidden (`walk.rs:862`, `874`). `overrides` (`walk.rs:833`) gives gitignore-syntax globs, a natural fit for `exclude_paths`. 15 crates in `cargo tree`. Verdict yes: 4.6 times faster than Go, no custom code, and it is already needed for untracked detection. It is 1.35 times slower than `std` sequentially (1,300 versus 962 ms) and 1.8 times slower than the best hand-rolled walker at 12 threads.
- Hand-rolled rayon with rustix `RawDir` (`Repos/` has no clone of rustix; source read in the cargo registry: `rustix-1.1.5/src/fs/raw_dir.rs:16-30`, zero allocation iteration with a caller buffer; the older `Dir` allocates a `CString` per entry and starts with a buffer of about 768 bytes, `src/backend/linux_raw/fs/dir.rs:80-250`, so use `RawDir`). 227 ms, 8.5 times faster than Go. About 60 lines. Verdict maybe: a measured upgrade path if the scan becomes the bottleneck, and `rayon` plus `rustix` are in the tree anyway.
- `walkdir` 2.5.0 (2024-03-01, Unlicense/MIT, 2 crates): sequential, 1.28 times slower than `std`. It is a dependency of `ignore` already. Verdict no for direct use (vault discovery can use `ignore` too).
- `jwalk` 0.9.0 (2026-08-05, MIT, 12 crates, `Repos/jwalk`): pruning works through `process_read_dir` and `read_children = None` (`jwalk/src/lib.rs:342-343`), but it is the slowest parallel option (639 ms at 12 threads, 4,127 ms CPU) and plateaus at 8 threads. Verdict no.
- Lower level: `d_type` is already free through `read_dir` and `getdents64` on ext4 (no stat in any variant). `io_uring` has no `getdents` operation: the installed `/usr/include/linux/io_uring.h` (linux-api-headers 6.16) lists `IORING_OP_NOP` through `IORING_OP_EPOLL_WAIT` and no directory-read opcode (grep for `GETDENTS` returns nothing). Not verified against the newest kernel tree, but `statx` via io_uring would only replace calls we do not make. Verdict no.

### Git without a subprocess

- `gix-index` 0.56.0 plus `gix-discover` plus `gix-hash` with `sha1` (MIT OR Apache-2.0, GitoxideLabs, pushed 2026-10-06, 12k stars, `Repos/gitoxide` at 9272d45). `File::at(path, Kind::Sha1, skip_hash, options)` memory-maps the index and optionally verifies the checksum (`gix-index/src/file/init.rs:64`). `prefixed_entries` (`gix-index/src/access/mod.rs:386-425`) takes a raw byte prefix: pass `skill/` with a trailing slash or `coding` also matches `coding-extra`. It keeps all stages of a conflicted path, so filter `stage_raw() == 0`. Exclude symlink entries (`Mode::SYMLINK`) and gitlinks, and treat `is_sparse()` as a hard error. `gix_discover::upwards` finds the repository above the vault, so a vault inside a larger repository works with a prefix; decide whether that is wanted. Verdict yes (0.10 ms, 83 crates, +0.6 MB stripped).
- `gix` high-level with `default-features = false, features = ["index", "sha1"]`: 128 crates, +1.5 MB. Adds repository config handling for no gain here. Verdict no. With `status` and `dirwalk` (`gix/Cargo.toml` features `status`, `dirwalk`): 161 crates, +2.2 MB, 346 s compile CPU, untracked detection 15.8 ms. The gitignore engine is the real thing, but `ignore` does the same job for free. Verdict maybe, only if exact `git status` semantics for collapsed untracked directories become a requirement.
- `git2` 0.21.0 (2026-05-18, MIT OR Apache-2.0 for the Rust part): 13 crates but compiles libgit2 from C (240 s CPU) and, decisively, statically links LGPL and GPL-with-exception code. See the licence section. Verdict no.
- Shelling out: one spawn per vault is 1.9 ms, one per skill 41 ms. The 395-spawn pattern is a Go design flaw (the file list depends on the source skill, not the target), fixable by caching in either language. The case for in-process is therefore robustness (no `git` binary, no PATH or locale effects, and git's "dubious ownership" refusal on repositories owned by another user, a known git behaviour I did not reproduce here), not speed. A single `git ls-files --cached -z` plus `git ls-files --others --exclude-standard -z` at start is the zero-dependency alternative; the requirement says no subprocess, so I recommend `gix-index`, and list the spawn as the fallback-free alternative for the lead to weigh (83 crates against two 2 to 10 ms spawns).
- Open semantic point: the Go code copies the working-tree file for each tracked path, not the staged blob. A tracked file with unstaged edits is copied as edited. Reading the blob from the object database would need more `gix` features. Decide which is intended; the Go behaviour is the cheaper one.

### Comparing and hashing

- Equality needs no hash: size differs means changed (safe direction), otherwise read and compare bytes. Measured 38 ms for a full no-op plan over 395 targets with 8 threads, against 27 ms for size only. A hash adds 2.4 times to that for nothing, because the vault side has to be read anyway.
- mtime and size fast path: not recommended. Risks: kernel timestamps are coarse (a file rewritten within one tick keeps the same mtime), tools that preserve mtime (`cp -p`, `rsync -t`, restores) keep old timestamps over new content, and a false "unchanged" is the dangerous direction. The gain is under 80 ms in the measured worst case. If a fast path is wanted later, use only "size differs".
- Hash needed only for the provenance lock file (is the installed file still what we installed). Candidates, dependency counts from `cargo tree`: `sha2` 0.11.0 (2026-03-25, MIT OR Apache-2.0, 10 crates, 1.0B downloads) verdict maybe, yes if the lock file is built, because humans can check it with `sha256sum`; `blake3` 1.8.7 (CC0-1.0 OR Apache-2.0, 8 crates, needs a C compiler step for SIMD, speed irrelevant at 2 MB) verdict no; `xxhash-rust` 0.8.19 (BSL-1.0, not on the allowed list, 1 crate) verdict no; `twox-hash` 2.1.5 (2026-10-04, MIT, 1 crate with `default-features = false`, features `xxhash3_64`) verdict maybe if a non-cryptographic 128-bit drift check is enough.

### Atomic apply, copies, backups

- Staging then `renameat2(RENAME_EXCHANGE)` through `rustix::fs::renameat_with(CWD, stage, CWD, dst, RenameFlags::EXCHANGE)` (`rustix` 1.1.5, 2026-09-16, 3 crates with the `fs` feature, 1.2B downloads) works for directories here. After the swap the old tree sits at the stage path, so moving it into the backup store is a rename when both are on one filesystem. For a new skill use `rename(stage, dst)` or `RenameFlags::NOREPLACE`. Cost: 1.4 times the delete-and-copy approach sequentially (781 versus 543 ms min), 1.35 times at 8 threads, and it never leaves a half skill. Unsupported filesystems (some network or FUSE mounts) return an error: report it, do not degrade silently.
- Stage location: next to the destination but not inside `.agents/skills/`, for example `.agents/.stage-<run>/`, because agents list `skills/` and could load a transient `SKILL.md`. Same filesystem is guaranteed unless `skills` itself is a mount; `EXDEV` has been seen to be reported correctly (os error 18).
- `tempfile` 3.27.0 (2026-03-11, 9 crates): `Builder::tempdir_in` gives cleanup on error paths; also the test dependency. Verdict yes. `NamedTempFile::persist` covers atomic single-file writes (config, registry).
- `atomic-write-file` 0.3.1 (2026-08-11, BSD-3-Clause, 11 crates, `nix` and `rand` inside, crash-tested, preserves mode): good crate for single files, but `tempfile` plus an explicit `sync_all` and rename is 25 lines and already needed. Verdict no.
- `fs-err` 3.3.2 (2026-10-04, 2 crates): puts the path into every I/O error, which the "errors reported, never swallowed" rule needs. Includes `symlink` wrappers. Verdict yes.
- `std::fs::copy`: calls `copy_file_range` (strace on this kernel), copies permission bits exactly. Verdict: use it. `reflink-copy` 0.1.30 (MIT/Apache-2.0, 6 crates, `Repos/reflink-copy`): on this ext4 the clone ioctl returns `EOPNOTSUPP` (probe). Reflinks only exist on btrfs and XFS; the `copy_file_range(2)` man page (line 225 of the installed page) says filesystems may implement reflink-style copy acceleration behind it, so `std::fs::copy` benefits there without a crate. I could not test btrfs or XFS here (no root): not measured. The whole vault is 2 MB, so no meaningful saving either way. Verdict no. `filetime` 0.2.29: `File::set_modified` is in `std` since 1.75, and no mtime is needed. Verdict no.
- Permission bits: `PermissionsExt` is only needed to assert or to set a mode; `std::fs::copy` already preserves it (probe above, 19 of the 338 vault files are 0755).
- Backup and undo: own store under `$XDG_STATE_HOME/<tool>/backups/<run-id>/<project-slug>/<skill>/` with a small JSON manifest, filled by renames (same filesystem) with a copy-and-remove step on `EXDEV`. `trash` 5.2.9 (2026-09-13, MIT, 10 crates including `chrono`, `Repos/trash-rs`): FreeDesktop trash with `list` and `restore_all` (`src/freedesktop.rs:74`, `372`), but it restores by original path and deletion time, has no notion of a run or project, and puts data in per-mount trash folders. Verdict maybe for the `delete` command only (user-visible recycle bin), no for sync backups.
- Windows: not needed (Linux only).

### Directory symlinks for other agent directories (`.claude/skills` to `.agents/skills`)

All primitives are in `std`; `linkprobe` ran them on this ext4:

- Relative target `../.agents/skills` computed with `pathdiff::diff_paths` 0.2.3 (2024-11-25, MIT/Apache-2.0, 1 crate, 200M downloads) or ten lines of component logic. Verdict maybe.
- Classify with `symlink_metadata` (never `metadata`, which follows): missing, real directory, other file, link with the expected `read_link`, link with another target, dangling link (target does not exist). All six were detected correctly.
- Create with `std::os::unix::fs::symlink`: it fails with `AlreadyExists` on a link, a real directory and a file, so creation is atomic and never clobbers. Eight racing threads: exactly one winner.
- Replace a wrong link atomically: `symlink(target, tmp)` next to it, then `rename(tmp, link)`; the old link is replaced atomically. The same call against a real directory (empty or not) fails with `EISDIR`, leaving the directory and siblings untouched, so the rule "never replace a real directory" is enforced by the kernel. `renameat2(NOREPLACE)` gives no-clobber for the temp-link route.
- A `d_type` walker sees the link as `is_symlink`, not `is_dir`, so it never descends it and never discovers a second copy of the skills. Apply must always work on the real `.agents/skills/<name>` path, never through the link.

### Diffing

- `similar` 3.2.0 (2026-08-17, Apache-2.0, mitsuhiko, 1,332 stars, zero dependencies, `Repos/similar`): unified diff, inline word and char diff (used by the TUI and web diff view), Myers and Patience. 1,955 us for a 67 KB file with four edits. Verdict yes.
- `imara-diff` 0.2.0 (2025-06-14, Apache-2.0, 3 deps, `Repos/imara-diff`): histogram, 193 us on the same input (10 times faster), unified output identical, API is lower level (`Diff::compute`, `UnifiedDiffConfig`). Verdict maybe if diffs ever become hot. `diffy` 0.5.2 (2026-08-31, MIT OR Apache-2.0, 2 deps): 592 us, patch parsing and applying, unused here. Verdict no.

### YAML: reading

Real error cases run against both candidates (`yamlbench`, a `deny_unknown_fields` struct):

| Case | `serde_yaml_ng` 0.10.0 | `serde-saphyr` 1.3.0 |
|---|---|---|
| unknown key | error, line and column | error, line and column plus source snippet |
| wrong type (`mandatory: 5`) | error, line and column | error with snippet |
| missing `root` | error without position | error with position and snippet |
| duplicate key | error | error with snippet, names the policy option |
| tab indentation | scanner error | error with snippet |

- `serde-saphyr` 1.3.0 (2026-09-16, MIT OR Apache-2.0, 228 stars, pushed 2026-10-06, 21 crates, `Repos/serde-saphyr`): YAML 1.2, panic-free design, configurable budgets, no tag-driven construction, `Commented<T>` for comment capture on read, serializer included. The clone's `Cargo.toml` already says 2.0.0 (unreleased on crates.io at the time of checking); pin 1.3. Verdict yes.
- `serde_yaml_ng` 0.10.0 (2024-05-26, MIT, 114 stars, last push 2025-09-14, 9 crates, `Repos/serde-yaml-ng`): works, but the last release is 2.4 years old and the errors are plainer. Verdict no. `serde_yaml` 0.9.34+deprecated: archived. No.
- `yaml-rust2` 0.13.0 (2026-09-11, 7 crates) and `saphyr` 0.1.0 (2026-09-19, 20 crates): untyped trees plus an emitter that drops comments, no serde. They are parser building blocks (`saphyr-parser` event spans could drive a span-based editor, not tested). Verdict no.

### YAML: comment-preserving edit versus TOML

Question: keep `config.yaml` and edit it in place, or move the new config to TOML with `toml_edit` and a migration command? Evidence:

- `yaml-edit` 0.3.2 (2026-09-17, Apache-2.0, Jelmer Vernooij, 7 crates with rowan, 10 stars, 143k downloads, minor versions every few weeks since 0.2.0 on 2026-03-05; `Repos/yaml-edit`). It is a lossless rowan syntax tree. What works (tested): `set` on a scalar keeps the trailing comment and all other comments, flow and block lists accept `push`, `remove(i)` keeps neighbouring comments, awkward strings are quoted correctly (20 values including `has: colon`, `# hash`, `yes`, `null`, `007`, `- dash`, an empty string: the round trip through a strict parser was equal). What breaks: `Mapping::set(key, Vec<YamlValue>)` on an existing key writes invalid YAML (`mandatory: - q1` then `- q2` on the next line; suspected cause: the `From<Vec<T>>` conversion at `src/value.rs:547` handing a block sequence to an inline slot, not isolated further). `Document::from_str` rejects a file that starts with a comment (`YamlFile::from_str` is required). `clear()` turns a block list into `[]` and stale handles must be fetched again. A pushed line in a CRLF file gets `\n`. A key with a null value (`mandatory:`) has no sequence handle.
- Randomised test, 5,000 generated configs (comments at head, between keys and inside lists, block and flow lists, quoted and awkward values, lists at indent 0 and 2): replace `root` and rewrite `mandatory` using only safe primitives (remove from the end, push), then require a strict parse, equal typed values, and all outer comments present. Result: 4,771 clean, 0 parse errors, 0 value mismatches, 229 cases lost an outer comment (a comment sitting directly under a replaced list is attached to the list and removed with it), and 669 lost comments inside the replaced list (expected).
- A 70-line line-based editor (`yamlbench/src/line_edit.rs`: find the top-level `key:` line, take following indented, dash or comment lines as the block, replace with a freshly rendered block, quote values through `serde-saphyr`, keep the old trailing comment on scalars, append when absent): the same 5,000 cases gave 5,000 clean with zero lost outer comments. Caveat: the generator and the editor share assumptions (flat top-level keys), so this proves the editor for the shapes the tool itself writes, not for arbitrary YAML. That is why every edit must be followed by re-parse and typed comparison, which turns any miss into a hard error and leaves the file untouched.
- `toml_edit` 0.25.15 (MIT OR Apache-2.0, 8 crates, 880M downloads): in-place edits keep comments; assigning a new value to a key drops that value's trailing comment, and replacing a whole array drops its inner comments (tested). Structurally safe (output is always valid), mature, but it forces a migration and a second parser: frontmatter in every `SKILL.md` is YAML, so no YAML dependency is saved.

Recommendation: keep `config.yaml`. Read with `serde-saphyr` into a `deny_unknown_fields` struct. Write with the small line editor (own code, about 70 lines, in its own module) followed by mandatory verification (re-parse, compare to the intended values, write via temp file and rename). `yaml-edit` is the maybe if the config ever becomes a nested document; adopt it only behind the same verification wrapper. TOML is plan C, not needed.

### Frontmatter

Tested on the 24 real `SKILL.md` files plus eight edge cases (CRLF, BOM, folded scalar, no frontmatter, `---` in the body, extra key, trailing spaces on the fence, unquoted colon).

- Manual split (file must begin with `---` line, end at the next line that is exactly `---`) plus `serde-saphyr`: 22 of 24 parse, 17 us per file. `gray_matter` 0.3.2 (2025-07-10, MIT, 25 crates): the same 22 of 24, 30 us, and it accepts a BOM or trailing spaces on the fence without reporting a missing block. `markdown-frontmatter` 0.5.1 (2026-03-03, MIT, 22k downloads, 30 crates): 22 of 24, fails on no-frontmatter files with a JSON error. Verdict: manual split, no crate (gray_matter no, markdown-frontmatter no).
- Real data finding: `post-scheduler/SKILL.md` and `secrets/SKILL.md` in the vault have an unquoted `description:` containing `: ` and fail all three parsers (`mapping values are not allowed in this context`, line 2 column 218 and 156). Under the "hard errors" rule the first sync after the rewrite would stop on two existing skills. The requirements need a decision: quote them in the vault (preferred, `doctor` reports the fix), or accept a lenient `name` and `description` line extractor with a visible warning.

### Config and data directories

`etcetera` 0.11.0 (2025-10-28, MIT OR Apache-2.0, 2 crates), `dirs` 7.0.0 (2026-09-05, 4 crates), `directories` 6.0.0 (2025-01-12, 4 crates) all agreed on this machine for config, data, state and cache, honour `XDG_CONFIG_HOME` and `XDG_STATE_HOME`, resolve `HOME` through the passwd database when unset, and ignore a relative `XDG_CONFIG_HOME`. Differences: `directories::ProjectDirs::from("", "", "skill_Manag")` lowercases the name to `skill_manag`, which differs from the existing `~/.config/skill_Manag` (migration trap); `dirs` and `directories` pull `dirs-sys` and `option-ext` (MPL-2.0). The Go code hard-codes `~/.config/skill_Manag/vault` and ignores `XDG_CONFIG_HOME`. Verdict: `etcetera` yes (or ten lines of `std`), `dirs` and `directories` no.

### Concurrency and events

Typed enum events (project found, plan ready, applied, error), 4 producers, 400,000 events, median of 5:

| Channel | median ms | million events/s at min |
|---|---|---|
| `std::sync::mpsc` unbounded | 107 | 3.8 |
| `std::sync::mpsc::sync_channel(1024)` | 114 | 3.6 |
| `crossbeam-channel` 0.5.17 unbounded | 72 | 6.1 |
| `crossbeam-channel` bounded(1024) | 135 | 3.9 |
| `flume` 0.12.0 unbounded | 208 | 2.4 |
| `flume` bounded(1024) | 574 | 0.8 |

A real run emits a few thousand events, so throughput is irrelevant. `std::sync::mpsc` has `recv_timeout` and `try_recv` (checked), is dependency free and enough for CLI, TUI tick loops and a web server thread. `crossbeam-channel` earns its place only if one loop must `select!` over engine events and another source (checked: `select!` works) or needs several consumers. `flume` is slower here and adds nothing. Parallelism: `rayon` 1.12.0 (2026-04-14, 6 crates) with `ThreadPoolBuilder::num_threads` for bounded apply and scan fan-out. Cancellation: a shared `AtomicBool` checked per directory or per target (round trip to a worker about 0 ms of added latency in the probe), `WalkState::Quit` in `ignore`. Tokio is not needed anywhere in the engine; the web view can run its server on a thread and read the same channel.

### Registry format, errors, paths

- Registry: `serde_json` 1.0.151 (5 crates) pretty-printed (28 KB, 0.2 ms read) is the simplest thing that works and stays diffable. `postcard` 1.1.3 (2025-07-24, 24 crates) is smaller but opaque for a file of 20 KB. `redb` 4.3.0 (2026-10-02, 2 crates) and `rusqlite` 0.40.2 (14 crates plus C) solve a problem this registry does not have. `bincode` 3.0.0 is a tombstone: its `lib.rs` is a `compile_error!` and its README says development ceased, so it must not be used. Verdict: `serde_json` yes.
- `thiserror` 2.0.21 (2026-09-23, 8 crates, proc-macro already compiled for serde): yes for the library crate's error enums. `fs-err`: yes (above). `camino` 1.2.6 (1 crate): no. Linux paths are bytes; `serde_json` already refuses a non-UTF-8 `PathBuf` with "path contains invalid UTF-8 characters" and `postcard` errors too (tested), which gives the hard error at the one boundary that matters, while `camino::Utf8PathBuf::from_path_buf` would have to be threaded through `ignore`, `rustix` and `std` results that all return `PathBuf`.

## Recommendation

Concrete set (versions checked 2026-10-07):

- Walk and scan: `ignore` 0.4.33 (`build_parallel`, standard filters off, `filter_entry` for noise, `Override` for `exclude_*`, 12 threads default and configurable), `rayon` 1.12.0. Optional later: rayon plus `rustix` `RawDir` walker (measured 1.8 times faster).
- Registry: `serde_json` 1.0.151, JSON file in the XDG state directory, written with `tempfile` persist; revalidate by `stat` of each skills directory, walk on `--rescan`, on a TTL, or when the root changes, streaming `ProjectFound` while cached projects display.
- Git: `gix-index` 0.56.0, `gix-discover`, `gix-hash` (sha1); untracked via `ignore` plus the index set; not a git repository is a hard error from `gix_discover::upwards`.
- Compare: `std` only (size then bytes). Lock file digest (only if built): `sha2` 0.11.0.
- Apply: `rustix` 1.1.5 (`renameat_with` `EXCHANGE` and `NOREPLACE`), `tempfile` 3.27.0, `fs-err` 3.3.2, `std::fs::copy`, `std::os::unix::fs::symlink`; own backup store with a JSON manifest.
- Diff: `similar` 3.2.0.
- Config: `serde-saphyr` 1.3.0 read, own line editor plus verification write, `etcetera` 0.11.0 for paths. Frontmatter: manual split plus `serde-saphyr`.
- Events and errors: `std::sync::mpsc` (add `crossbeam-channel` 0.5.17 only if `select!` is needed), `thiserror` 2.0.21.
- Pinning note: `gix` crates move together on 0.x versions every few weeks; pin the three in one place and update them as a group.

## Rejected and why

- `git2`: LGPL and GPL-with-exception code compiled in, C build, no speed gain (see licence section).
- `jwalk`: slowest parallel walker here, 12 crates.
- `walkdir` (direct), `io_uring`, `nix`: covered by `ignore`, no directory-read opcode, `rustix` chosen (`nix` 0.31.3, MIT, 5 crates, is a dependency of `atomic-write-file`).
- `blake3`, `xxhash-rust`, `twox-hash` for equality: not needed; `blake3` brings a C step, `xxhash-rust` is BSL-1.0.
- `atomic-write-file`, `reflink-copy`, `filetime`: replaced by `tempfile`, `std::fs::copy`, `File::set_modified`.
- `serde_yaml` (archived), `serde_yaml_ng` (last release 2024-05-26), `yaml-rust2`, `saphyr` (no comment preservation, no serde), `gray_matter`, `markdown-frontmatter` (no gain over a manual split).
- `dirs`, `directories`: MPL-2.0 transitive and lowercase-name trap.
- `flume`, `postcard`, `redb`, `rusqlite`, `bincode` 3.0.0 (tombstone with `compile_error!`), `camino`, `diffy`, `trash` for backups.
- Tokio in the engine.

## Good but licence-blocked

Project licence: Hippocratic License 3.0. Census: `cargo metadata --filter-platform x86_64-unknown-linux-gnu` over a project that depends on every candidate in this file (243 packages): 169 `MIT OR Apache-2.0`, 23 `MIT`, 11 `MIT/Apache-2.0`, 8 `Unlicense OR MIT`, 7 `Apache-2.0 OR MIT`, 5 `Apache-2.0`, 3 `Zlib`, plus single crates under `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`, `CC0-1.0 OR MIT-0 OR Apache-2.0`, `CC0-1.0 OR Apache-2.0 ...` (blake3), `BSD-3-Clause` (`atomic-write-file`), `(Apache-2.0 OR MIT) AND BSD-3-Clause` (`encoding_rs`), `(MIT OR Apache-2.0) AND Unicode-3.0` (`unicode-ident`), `Zlib OR Apache-2.0 OR MIT` (`tinyvec`), `BSL-1.0` (`xxhash-rust`), `Apache-2.0 OR BSL-1.0` (`ryu`), `MPL-2.0` (`option-ext`). This reads the `license` field of each package; I did not run `cargo deny` and did not read every vendored licence file.

- `git2` 0.21.0 via `libgit2-sys` 0.18.8: metadata says `MIT OR Apache-2.0` but that covers only the Rust glue. The vendored C library is GPL-2.0 with a linking exception (`libgit2-sys-0.18.8+1.9.7/libgit2/COPYING` lines 1 to 20), bundles `deps/xdiff` under LGPL-2.1-or-later (`deps/xdiff/xdiff.h` lines 1 to 17), and winhttp definition files under LGPL (COPYING line 513, Windows only). `libgit2-sys/build.rs:162-164` compiles `xdiff` into the static library, also with `default-features = false`. Static LGPL linking is a blocker until reviewed. Strong candidate otherwise (stable, fast enough, 13 crates). Not needed since `gix-index` does the job.
- `dirs` 7.0.0 and `directories` 6.0.0: permissive themselves, but pull `dirs-sys` and `option-ext` 0.2.0, `MPL-2.0` (allowed, file-level copyleft; marked). `etcetera` does not.
- `xxhash-rust` 0.8.19: `BSL-1.0` (Boost, permissive but not on the list in the requirements; not recommended anyway).
- GPL, AGPL, SSPL, BUSL, source-available or non-commercial licences among the crates I looked at: none found. Licences I checked: MIT, Apache-2.0 (with and without LLVM exception), Unlicense, Zlib, BSD-3-Clause, CC0-1.0, MIT-0, BSL-1.0, Unicode-3.0, MPL-2.0, and for `libgit2` the files named above.
- Does any permissive crate in the recommended set pull a copyleft or odd transitive licence? No GPL or LGPL: the recommended set (`ignore`, `gix-index`, `serde-saphyr`, `rustix`, `tempfile`, `fs-err`, `similar`, `etcetera`, `thiserror`, `rayon`, `serde_json`) contains only MIT, Apache-2.0, Unlicense (dual with MIT), Zlib (`zlib-rs`, `foldhash` via gix), BSD-3-Clause (`encoding_rs` via `serde-saphyr`), Unicode-3.0 (`unicode-ident`) and CC0 or Apache (`constant_time_eq`). `gix` itself had no odd licence in its 161-crate tree.

## Risks and open points

1. Measurement quality: loadavg 8 to 33 during all timings; ratios are reliable, absolutes are not. Cold cache not measured. Re-run `run_matrix_c.sh` on an idle machine and after `drop_caches`.
2. `yaml-edit` list replacement bug and 0.x churn (above); the line editor needs the verification step to be trusted for files shapes beyond the tool's own.
3. Two real `SKILL.md` files fail strict YAML frontmatter; policy decision needed.
4. Working-tree versus staged-blob copy semantics; vault nested in a larger repository; sparse index and gitlinks rejected as hard errors.
5. `RENAME_EXCHANGE` unsupported on some filesystems; handled as an error.
6. Registry staleness: it can miss new projects until a rescan; a TTL and `--rescan` bound that.
7. `gix` 0.x API churn: three crates to pin and bump together.
8. Not verified: git "dubious ownership" behaviour on this machine, reflink on btrfs or XFS, cold walk, `saphyr-parser` span-based editor, `serde-saphyr` 2.0.

## Changes suggested for the requirements doc

- Baseline: the Go walk alone is 1.3 to 1.9 s and the 395 spawns another 0.5 s at idle; "plain find 1.6 s" does not reproduce with GNU find (4.4 to 6.6 s); name the tool used. The tree grows while agents clone into `Repos/` inside the scan root, which changes the baseline.
- The registry must record every `.agents/skills` directory (65 to 71), not only the 60 projects that have a matching skill.
- Plan: drop "strong hash" from the content comparison; hash only for the lock file. Drop the mtime fast path.
- Apply: stage in `.agents/.stage-<run>/`, never inside `skills/`, and never apply through a symlinked agent directory.
- Frontmatter: decide on the two invalid `SKILL.md` files.
- Config path: honour `XDG_CONFIG_HOME` and keep the old `skill_Manag` spelling for migration.
- File selection: state whether the working-tree file or the staged blob is the copy source.

## Clones

Made by me in `Repos/` (shallow, commit): `gitoxide` (9272d45), `git2-rs` (f6f1116), `jwalk` (a5b1ea6), `walkdir` (6fd031c), `reflink-copy` (7886ede), `atomic-write-file` (e1ee3a5), `similar` (96993d9), `imara-diff` (f02e46a), `diffy` (4aa5870), `saphyr` (a9c9154), `serde-saphyr` (8d80974), `yaml-rust2` (e120694), `yaml-edit` (ae2303a), `serde-yaml-ng` (3628102), `gray-matter-rs` (94967cb), `markdown-frontmatter` (4293da2), `trash-rs` (b8d9ebc), `toml-rs` (9750f43), `tempfile` (1c294cc), `BLAKE3` (f55849f). Reused from another agent: `ripgrep`. Read in depth: `gitoxide` (`gix-index`, `gix`), `ripgrep/crates/ignore`, `jwalk`, `yaml-edit`, `serde-saphyr`, `atomic-write-file`, `trash-rs`, `imara-diff`. Only skimmed or unused: `diffy`, `saphyr`, `yaml-rust2`, `serde-yaml-ng`, `gray-matter-rs`, `markdown-frontmatter`, `toml-rs`, `tempfile`, `BLAKE3`, `git2-rs`, `reflink-copy`, `similar`, `walkdir`. Crate sources of `rustix`, `libgit2-sys` and `bincode` were read in `~/.cargo/registry/src`.

## Verdict table

| Crate | Verdict | Reason | Version checked | Licence |
|---|---|---|---|---|
| ignore | yes | parallel walk 4.6 times faster than Go, overrides, also does untracked detection | 0.4.33 | Unlicense OR MIT |
| rayon | yes | bounded parallelism for scan and apply | 1.12.0 | MIT OR Apache-2.0 |
| rustix | yes | RENAME_EXCHANGE and NOREPLACE, optional RawDir walker | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| walkdir | no | sequential, already inside ignore | 2.5.0 | Unlicense/MIT |
| jwalk | no | slowest parallel walker, 12 crates | 0.9.0 | MIT |
| gix-index (+ gix-discover, gix-hash) | yes | 0.10 ms index read, 83 crates, +0.6 MB | 0.56.0 | MIT OR Apache-2.0 |
| gix (index only) | no | 128 crates for no gain over gix-index | 0.88.0 | MIT OR Apache-2.0 |
| gix (status, dirwalk) | maybe | exact git status semantics, 161 crates, 346 s compile CPU | 0.88.0 | MIT OR Apache-2.0 |
| git2 | no | libgit2 C build with GPL-with-exception and LGPL xdiff | 0.21.0 | MIT OR Apache-2.0 (Rust glue only) |
| sha2 | maybe | lock file digest if the lock file is built | 0.11.0 | MIT OR Apache-2.0 |
| blake3 | no | speed irrelevant at 2 MB, C step | 1.8.7 | CC0-1.0 OR Apache-2.0 |
| xxhash-rust | no | BSL-1.0, no need | 0.8.19 | BSL-1.0 |
| twox-hash | maybe | 1 crate non-cryptographic alternative for the lock | 2.1.5 | MIT |
| tempfile | yes | stage directories with cleanup, atomic file persist | 3.27.0 | MIT OR Apache-2.0 |
| fs-err | yes | path in every I/O error | 3.3.2 | MIT OR Apache-2.0 |
| atomic-write-file | no | tempfile plus rename suffices, 11 crates | 0.3.1 | BSD-3-Clause |
| reflink-copy | no | ext4 unsupported, std copy_file_range, 2 MB vault | 0.1.30 | MIT/Apache-2.0 |
| filetime | no | std File::set_modified | 0.2.29 | MIT/Apache-2.0 |
| trash | maybe | only for a user-facing delete, not for backups | 5.2.9 | MIT |
| pathdiff | maybe | relative symlink targets, or ten lines | 0.2.3 | MIT/Apache-2.0 |
| nix | no | rustix chosen | 0.31.3 | MIT |
| similar | yes | zero deps, unified and inline diff | 3.2.0 | Apache-2.0 |
| imara-diff | maybe | 10 times faster histogram diff if ever hot | 0.2.0 | Apache-2.0 |
| diffy | no | no advantage | 0.5.2 | MIT OR Apache-2.0 |
| serde-saphyr | yes | strict typed YAML with snippets, active | 1.3.0 | MIT OR Apache-2.0 |
| serde_yaml_ng | no | last release 2024-05-26 | 0.10.0 | MIT |
| serde_yaml | no | archived | 0.9.34+deprecated | MIT OR Apache-2.0 |
| yaml-edit | maybe | works for safe operations, list replacement bug, 0.x | 0.3.2 | Apache-2.0 |
| saphyr / yaml-rust2 | no | untyped trees, comments lost | 0.1.0 / 0.13.0 | MIT OR Apache-2.0 |
| toml_edit | maybe | plan C if YAML editing is dropped | 0.25.15 | MIT OR Apache-2.0 |
| gray_matter | no | no gain over manual split, 25 crates | 0.3.2 | MIT |
| markdown-frontmatter | no | errors on missing frontmatter, 30 crates | 0.5.1 | MIT |
| etcetera | yes | XDG paths, 2 crates, no MPL | 0.11.0 | MIT OR Apache-2.0 |
| dirs / directories | no | MPL-2.0 via option-ext, name lowercasing | 7.0.0 / 6.0.0 | MIT OR Apache-2.0 |
| crossbeam-channel | maybe | only if select over several sources is needed | 0.5.17 | MIT OR Apache-2.0 |
| flume | no | slower, no benefit | 0.12.0 | Apache-2.0/MIT |
| thiserror | yes | library error enums | 2.0.21 | MIT OR Apache-2.0 |
| camino | no | PathBuf boundaries everywhere, serde already rejects non-UTF-8 | 1.2.6 | MIT OR Apache-2.0 |
| serde_json | yes | registry file | 1.0.151 | MIT OR Apache-2.0 |
| postcard | no | opaque, no benefit at 20 KB | 1.1.3 | MIT OR Apache-2.0 |
| redb / rusqlite | no | overkill | 4.3.0 / 0.40.2 | MIT OR Apache-2.0 / MIT |
| bincode | no | tombstone release, compile_error | 3.0.0 | MIT |
