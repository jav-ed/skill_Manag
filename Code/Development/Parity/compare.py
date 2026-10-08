#!/usr/bin/env python3
"""compare.py - compare golden/ (Go oracle) with parity/out/ (skillmirror) per scenario and step.

  compare.py [--golden DIR] [--out DIR] [--divergences FILE] [--diagnoses FILE]
             [--observations FILE] [--report FILE] [--json FILE] [--previous FILE]

Defaults come from the work directory: $PARITY_WORK (default <repo>/Scratch/Oracle, found from this
script's path) holds golden/ and parity/{out,report.md,results.json}; the divergence table is the
tracked Project_Manag/Docs/Investigation/Parity_Oracle/divergences.tsv. diagnoses.tsv and
observations.md are optional files in $PARITY_WORK/parity/ (missing means empty).

Checks per step (all four are computed, none is skipped):
  tree   projects.manifest golden == out, byte for byte
  exit   Rust exit code is in the set that the exit-code table maps the Go exit code to
  set    {(project, skill, file-count)} of Go's dry-run text / driver lines == the entries of
         `skillmirror <sync|push> --dry-run --json --all` (plan.json); n/a when Rust printed no JSON
and per scenario:
  rest   rest.manifest golden == out, ignoring everything below home/ (the config dirs differ by design)

Classification of a scenario:
  MATCH                 every check of every step agrees
  EXPECTED-DIVERGENCE   some check differs and an entry in the divergences file explains EVERY
                        differing check (its `covers` list names the check kinds, optionally with
                        a constraint on the Rust result, e.g. exit=3 or tree=pre)
  UNEXPECTED            anything else: a differing check without an entry, or an entry whose
                        constraint the Rust result violates
  SKIPPED               steps that could not be translated (none expected)

Exit-code table Go -> Rust (function expected_rust_exit):
  Go 0, no error reported            -> {0}
  Go 0 but errors reported           -> {3, 4}   (driver ERR line, delete line on stderr, dry-run error line)
  Go 1, usage error (cobra text)     -> {2}
  Go 1, anything else                -> {3}
  Go 124 (timeout)                   -> {124}
"""
import argparse
import difflib
import json
import os
import re
import shlex
import sys

USAGE_PAT = re.compile(r"^Error: (unknown (flag|command|shorthand)|accepts at most|flag needs|required flag)")


def read(path, default=""):
    try:
        with open(path, encoding="utf-8", errors="replace") as fh:
            return fh.read()
    except OSError:
        return default


def lines(path):
    return read(path).splitlines()


def expected_rust_exit(go_exit, go_stdout, go_stderr, go_kind):
    """Documented mapping, see module docstring."""
    if go_exit == 124:
        return {124}
    if go_exit == 0:
        errs = (
            (go_kind == "drv" and re.search(r"^ERR\t", go_stdout, re.M))
            or re.search(r"^\s+✗ .*", go_stderr, re.M)
        )
        return {3, 4} if errs else {0}
    first = go_stderr.splitlines()[0] if go_stderr.strip() else ""
    if USAGE_PAT.match(first):
        return {2}
    return {3}


def norm_project(p):
    """Compare project paths independent of how the root was written (absolute, relative, or through the
    fixture's `projects_link` symlink: the Rust tool prints the canonical root)."""
    for prefix in ("<TMP>/projects/", "<TMP>/projects_link/", "projects/"):
        if p.startswith(prefix):
            return p[len(prefix):]
    return p


def go_set(kind, args, stdout):
    """(project, skill, files) from Go's dry-run text or the driver report; None if the step has none."""
    if kind == "drv":
        res = set()
        for ln in stdout.splitlines():
            m = re.match(r"^(ok|ERR)\t([^\t]+)\t([^\t]*)\tfiles=(\d+) ", ln)
            if m:
                res.add((norm_project(m.group(2)), m.group(3), int(m.group(4))))
        return res
    if args and args[0] == "--dry-run":
        res, proj = set(), None
        for ln in stdout.splitlines():
            if ln.startswith("● "):
                proj = ln[2:]
                continue
            m = re.match(r"^  ~ (.*?)\s+would sync \((\d+) files\)$", ln)
            if m and proj is not None:
                res.add((norm_project(proj), m.group(1), int(m.group(2))))
        return res
    return None


def rust_set(plan_path, plan_code_path):
    code = read(plan_code_path).strip()
    if not code:
        return None, "no plan call"
    if code != "0":
        return None, "plan exit %s" % code
    try:
        doc = json.loads(read(plan_path))
    except ValueError as exc:
        return None, "plan not JSON: %s" % exc
    res = set()
    for e in doc.get("entries", []):
        res.add((norm_project(e.get("project", "")), e.get("skill", ""), int(e.get("files") or 0)))
    return res, ""


def manifest_diff(a, b, limit=24):
    d = list(difflib.unified_diff(a, b, "golden", "rust", lineterm="", n=0))
    d = [x for x in d if not x.startswith(("---", "+++", "@@"))]
    more = len(d) - limit
    return d[:limit] + (["... %d more" % more] if more > 0 else [])


def load_tsv(path):
    rows = {}
    for ln in read(path).splitlines():
        if not ln.strip() or ln.startswith("#"):
            continue
        parts = ln.split("\t")
        while len(parts) < 4:
            parts.append("")
        rows[parts[0]] = parts[1:4]
    return rows


def covers_ok(cover, kind, step):
    """Does one `covers` token explain a differing check of this kind on this step?"""
    name, _, val = cover.partition("=")
    if name != kind:
        return False
    if not val:
        return True
    if kind == "exit":
        return str(step["rust_exit"]) in val.split("|")
    if kind == "tree":
        if val == "pre":
            return step["tree_vs_pre"]
        return True
    return True


def classify(steps, rest_ok, entry):
    diffs = [(s, k) for s in steps for k in ("tree", "exit", "set") if s["checks"][k] == "DIFF"]
    if not rest_ok:
        diffs.append((None, "rest"))
    if not diffs:
        return "MATCH", []
    if entry is None:
        return "UNEXPECTED", ["no divergence entry"]
    covers = [c.strip() for c in entry[1].split(",") if c.strip()]
    unexplained = []
    for s, kind in diffs:
        probe = s if s is not None else {"rust_exit": "", "tree_vs_pre": True}
        if not any(covers_ok(c, kind, probe) for c in covers):
            unexplained.append("%s %s" % (s["name"] if s else "scenario", kind))
    return ("EXPECTED-DIVERGENCE", []) if not unexplained else ("UNEXPECTED", unexplained)


def main():
    code = os.path.dirname(os.path.abspath(__file__))
    repo = os.path.normpath(os.path.join(code, "..", "..", ".."))
    work = os.environ.get("PARITY_WORK") or os.path.join(repo, "Scratch", "Oracle")
    par = os.path.join(work, "parity")
    ap = argparse.ArgumentParser()
    ap.add_argument("--golden", default=os.path.join(work, "golden"))
    ap.add_argument("--out", default=os.path.join(par, "out"))
    ap.add_argument("--divergences", default=os.environ.get("PARITY_DIVERGENCES") or os.path.join(
        repo, "Project_Manag", "Docs", "Investigation", "Parity_Oracle", "divergences.tsv"))
    ap.add_argument("--diagnoses", default=os.path.join(par, "diagnoses.tsv"))
    ap.add_argument("--observations", default=os.path.join(par, "observations.md"),
                    help="free text copied into the report (findings that are not parity failures)")
    ap.add_argument("--report", default=os.path.join(par, "report.md"))
    ap.add_argument("--json", default=os.path.join(par, "results.json"))
    ap.add_argument("--previous", default=os.path.join(par, "results_previous.json"),
                    help="results.json of the previous round; adds a 'changes since' section when it exists")
    a = ap.parse_args()
    if not os.path.isdir(a.golden):
        sys.exit("compare.py: no golden tree at %s (run run_Scenarios.sh first)" % a.golden)
    if not os.path.isdir(a.out):
        sys.exit("compare.py: no parity output at %s (run run_Parity.sh first)" % a.out)
    os.makedirs(os.path.dirname(os.path.abspath(a.report)), exist_ok=True)

    div = load_tsv(a.divergences)       # scenario -> [ref, covers, reason]
    diag = load_tsv(a.diagnoses)        # scenario -> [kind, text, ""]
    # divergence rows are  scenario<TAB>ref<TAB>covers<TAB>reason  -> stored as [ref, covers, reason]
    scenarios = sorted(d for d in os.listdir(a.golden) if os.path.isdir(os.path.join(a.golden, d)))
    results = []

    for scen in scenarios:
        g, o = os.path.join(a.golden, scen), os.path.join(a.out, scen)
        if not os.path.isdir(o):
            results.append({"scenario": scen, "class": "UNEXPECTED", "why": ["not run"], "steps": [], "desc": read(os.path.join(g, "DESCRIPTION")).strip()})
            continue
        steps = []
        for sd in sorted(x for x in os.listdir(g) if re.match(r"\d\d_", x)):
            gs, os_ = os.path.join(g, sd), os.path.join(o, sd)
            st = {"name": sd, "checks": {}, "notes": []}
            if os.path.exists(os.path.join(os_, "SKIPPED")):
                st["skipped"] = read(os.path.join(os_, "SKIPPED")).strip()
                st["checks"] = {"tree": "SKIP", "exit": "SKIP", "set": "SKIP"}
                st.update(rust_exit="", tree_vs_pre=False)
                steps.append(st)
                continue
            cmdline = shlex.split(read(os.path.join(gs, "cmd")).splitlines()[0]) if read(os.path.join(gs, "cmd")) else []
            go_kind = "drv" if cmdline and cmdline[0] == "skill_go_drv" else "go"
            go_args = cmdline[1:]
            go_exit = int(read(os.path.join(gs, "exit_code")).strip() or -1)
            go_out, go_err = read(os.path.join(gs, "stdout")), read(os.path.join(gs, "stderr"))
            r_exit = int(read(os.path.join(os_, "exit_code")).strip() or -1)
            st.update(go_exit=go_exit, rust_exit=r_exit, go_kind=go_kind, go_cmd=" ".join(cmdline))
            st["rust_cmd"] = read(os.path.join(os_, "cmd")).strip()
            st["go_stdout"], st["go_stderr"] = go_out, go_err
            st["rust_stdout"], st["rust_stderr"] = read(os.path.join(os_, "stdout")), read(os.path.join(os_, "stderr"))

            # tree
            gm, om = lines(os.path.join(gs, "projects.manifest")), lines(os.path.join(os_, "projects.manifest"))
            st["checks"]["tree"] = "OK" if gm == om else "DIFF"
            if gm != om:
                st["tree_diff"] = manifest_diff(gm, om)
            pre = lines(os.path.join(os_, "pre.manifest"))
            st["tree_vs_pre"] = om == pre
            # exit
            want = expected_rust_exit(go_exit, go_out, go_err, go_kind)
            st["exit_expected"] = sorted(want)
            st["checks"]["exit"] = "OK" if r_exit in want else "DIFF"
            # set
            gset = go_set(go_kind, go_args, go_out)
            if gset is None:
                st["checks"]["set"] = "n/a"
            else:
                rset, why = rust_set(os.path.join(os_, "plan.json"), os.path.join(os_, "plan_exit_code"))
                if rset is None:
                    st["checks"]["set"] = "n/a"
                    st["notes"].append("set not compared: " + why)
                else:
                    st["checks"]["set"] = "OK" if gset == rset else "DIFF"
                    if gset != rset:
                        st["set_diff"] = {
                            "only_go": sorted(gset - rset)[:12],
                            "only_rust": sorted(rset - gset)[:12],
                        }
            steps.append(st)

        grest = [x for x in lines(os.path.join(g, "rest.manifest")) if not re.search(r" home/", x)]
        orest = [x for x in lines(os.path.join(o, "rest.manifest")) if not re.search(r" home/", x)]
        rest_ok = grest == orest
        entry = div.get(scen)
        cls, unexplained = classify(steps, rest_ok, entry)
        if all(s.get("skipped") for s in steps) and steps:
            cls = "SKIPPED"
        res = {
            "scenario": scen, "class": cls, "why": unexplained, "steps": steps,
            "desc": read(os.path.join(g, "DESCRIPTION")).strip(),
            "divergence": entry, "diagnosis": diag.get(scen),
            "rest_ok": rest_ok,
        }
        if not rest_ok:
            res["rest_diff"] = manifest_diff(grest, orest)
        if cls == "MATCH" and entry is not None:
            res["note"] = "listed in divergences.tsv but every check matched (entry may be stale)"
        results.append(res)

    write_report(a, results, load_previous(a.previous))
    with open(a.json, "w") as fh:
        json.dump(results, fh, indent=1, default=str)

    counts = {}
    for r in results:
        counts[r["class"]] = counts.get(r["class"], 0) + 1
    print("scenarios: %d  %s" % (len(results), "  ".join("%s=%d" % kv for kv in sorted(counts.items()))))
    for r in results:
        if r["class"] == "UNEXPECTED":
            print("UNEXPECTED %s: %s" % (r["scenario"], "; ".join(r["why"])))
    return 0


def fence(text, n=14):
    ls = text.rstrip("\n").splitlines()
    if len(ls) > n:
        ls = ls[:n] + ["... (%d more lines)" % (len(ls) - n)]
    return "```\n" + "\n".join(ls) + "\n```"


def load_previous(path):
    try:
        with open(path) as fh:
            return {r["scenario"]: r["class"] for r in json.load(fh)}
    except (OSError, ValueError):
        return None


def changes_section(results, prev):
    """Per-scenario class transitions against the previous round, plus the count table."""
    cur = {r["scenario"]: r["class"] for r in results}
    order = ("MATCH", "EXPECTED-DIVERGENCE", "UNEXPECTED", "SKIPPED")
    out = ["## Changes since the previous round", ""]
    out.append("| class | previous | now |")
    out.append("|---|---|---|")
    for k in order:
        out.append("| %s | %d | %d |" % (k, sum(1 for v in prev.values() if v == k), sum(1 for v in cur.values() if v == k)))
    out.append("| **total** | **%d** | **%d** |" % (len(prev), len(cur)))
    out.append("")
    moved = [(s, prev[s], cur[s]) for s in sorted(cur) if s in prev and prev[s] != cur[s]]
    new = [s for s in sorted(cur) if s not in prev]
    out.append("Scenarios whose class changed (%d):" % len(moved))
    out.append("")
    for s, a, b in moved:
        out.append("- `%s`: %s -> %s" % (s, a, b))
    out.append("")
    out.append("New scenarios (%d):" % len(new))
    out.append("")
    for s in new:
        out.append("- `%s`: %s" % (s, cur[s]))
    out.append("")
    return out


def write_report(a, results, prev=None):
    counts = {}
    for r in results:
        counts[r["class"]] = counts.get(r["class"], 0) + 1
    ver = read(os.path.join(a.out, "RUST_VERSION")).strip()
    sha = read(os.path.join(a.out, "RUST_BIN_SHA256")).strip()
    out = ["# Parity report: Go oracle (c7310f9) vs skillmirror", ""]
    src = read(os.path.join(a.out, "RUST_SOURCE_SHA256")).strip()
    out.append("Binary under test: `%s` (sha256 `%s`), built from sources with sha256 `%s` (Crates/, Cargo.toml, Cargo.lock)"
               % (ver or "unknown", sha[:16] or "unknown", src[:16] or "unknown"))
    out.append("")
    out.append("| class | scenarios |")
    out.append("|---|---|")
    for k in ("MATCH", "EXPECTED-DIVERGENCE", "UNEXPECTED", "SKIPPED"):
        out.append("| %s | %d |" % (k, counts.get(k, 0)))
    out.append("| **total** | **%d** |" % len(results))
    out.append("")
    out.append("What is compared per step: the projects tree after the call (exact), the exit code (through "
               "the Go-to-Rust table in `compare.py`), and the set of (project, skill, file-count) from Go's "
               "dry-run text / driver lines against `skillmirror <sync|push> --dry-run --json --all`. Per scenario: "
               "everything outside `projects/` and `home/` (vault files, sentinel files). Output text, help text "
               "and the config directory differ on purpose and are not compared.")
    out.append("")
    out.append("Exit-code table (Go to Rust): Go 0 and no error reported -> 0; Go 0 with an error reported "
               "(driver `ERR` line, delete error line on stderr) -> 3 or 4; Go 1 with a cobra usage error -> 2; "
               "Go 1 otherwise -> 3. Rust codes: 0 clean, 1 drift with --check, 2 usage, 3 hard error, 4 partial.")
    out.append("")
    steps = [s for r in results for s in r["steps"]]
    cmp_set = sum(1 for s in steps if s["checks"].get("set") in ("OK", "DIFF"))
    na_set = sum(1 for s in steps if s["checks"].get("set") == "n/a")
    out.append("Coverage: %d steps; tree compared in %d, exit code in %d, (project, skill, file-count) set in %d "
               "(%d steps have no Go set or no Rust plan JSON)." % (
                   len(steps), sum(1 for s in steps if s["checks"].get("tree") in ("OK", "DIFF")),
                   sum(1 for s in steps if s["checks"].get("exit") in ("OK", "DIFF")), cmp_set, na_set))
    out.append("")

    if prev:
        out.extend(changes_section(results, prev))
    unexp = [r for r in results if r["class"] == "UNEXPECTED"]
    out.append("## UNEXPECTED (%d)" % len(unexp))
    out.append("")
    if not unexp:
        out.append("None.")
        out.append("")
    for r in unexp:
        out.extend(detail(r))
    exp = [r for r in results if r["class"] == "EXPECTED-DIVERGENCE"]
    out.append("## EXPECTED-DIVERGENCE (%d)" % len(exp))
    out.append("")
    out.append("| scenario | reference | differs in | reason |")
    out.append("|---|---|---|---|")
    for r in exp:
        kinds = sorted({k for s in r["steps"] for k, v in s["checks"].items() if v == "DIFF"} | ({"rest"} if not r["rest_ok"] else set()))
        ref, _, reason = (r["divergence"] or ["", "", ""])[0], "", (r["divergence"] or ["", "", ""])[2]
        out.append("| `%s` | %s | %s | %s |" % (r["scenario"], ref, ", ".join(kinds), reason))
    out.append("")
    skipped = [r for r in results if r["class"] == "SKIPPED" or any(s.get("skipped") for s in r["steps"])]
    out.append("## Skipped (%d)" % len(skipped))
    out.append("")
    if not skipped:
        out.append("None: every step could be translated (the Rust CLI is non-interactive).")
    for r in skipped:
        out.append("- `%s`: %s" % (r["scenario"], "; ".join(s.get("skipped", "") for s in r["steps"] if s.get("skipped"))))
    out.append("")
    stale = [r for r in results if r.get("note")]
    if stale:
        out.append("## Entries that matched anyway")
        out.append("")
        for r in stale:
            out.append("- `%s`: %s" % (r["scenario"], r["note"]))
        out.append("")
    match = [r for r in results if r["class"] == "MATCH"]
    out.append("## MATCH (%d)" % len(match))
    out.append("")
    out.append(", ".join("`%s`" % r["scenario"] for r in match))
    out.append("")
    obs = read(a.observations).strip()
    if obs:
        out.append("## Observations (not parity failures)")
        out.append("")
        out.append(obs)
        out.append("")
    with open(a.report, "w") as fh:
        fh.write("\n".join(out) + "\n")


def detail(r):
    out = ["### `%s`" % r["scenario"], "", r["desc"], ""]
    if r["why"]:
        out.append("Unexplained: " + "; ".join(r["why"]))
        out.append("")
    dg = r.get("diagnosis")
    out.append("**Diagnosis** (%s): %s" % ((dg[0], dg[1]) if dg else ("unclassified", "not yet analysed")))
    out.append("")
    for s in r["steps"]:
        bad = [k for k, v in s["checks"].items() if v == "DIFF"]
        if not bad:
            continue
        out.append("Step `%s` differs in: %s" % (s["name"], ", ".join(bad)))
        out.append("")
        out.append("Go call: `%s`  (exit %s, mapped to Rust %s)" % (s.get("go_cmd", ""), s.get("go_exit"), s.get("exit_expected")))
        out.append("")
        out.append("Rust call:")
        out.append(fence(s.get("rust_cmd", ""), 6))
        out.append("")
        out.append("Rust exit %s. Go stdout / Rust stdout (heads):" % s.get("rust_exit"))
        out.append("")
        out.append(fence(s.get("go_stdout", ""), 8))
        out.append(fence(s.get("rust_stdout", ""), 8))
        if s.get("rust_stderr", "").strip():
            out.append("Rust stderr:")
            out.append(fence(s["rust_stderr"], 6))
        if s.get("go_stderr", "").strip():
            out.append("Go stderr:")
            out.append(fence(s["go_stderr"], 4))
        if "tree_diff" in s:
            out.append("Tree diff (golden `-`, Rust `+`):")
            out.append(fence("\n".join(s["tree_diff"]), 24))
        if "set_diff" in s:
            out.append("Set diff: " + json.dumps(s["set_diff"], ensure_ascii=False))
        out.append("")
    if r.get("rest_diff"):
        out.append("rest.manifest diff:")
        out.append(fence("\n".join(r["rest_diff"]), 16))
        out.append("")
    return out


if __name__ == "__main__":
    sys.exit(main())
