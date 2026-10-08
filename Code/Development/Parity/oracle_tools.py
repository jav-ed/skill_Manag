#!/usr/bin/env python3
"""Helpers for run_Scenarios.sh (kept in Python so path handling is byte-exact).

  oracle_tools.py manifest DIR [--skip-top NAME ...] [--skip-name NAME ...] [--tmp BASE]
      --tmp BASE: hash file contents with BASE replaced by <TMP> (fixture paths inside files).
      Print one sorted line per entry below DIR:
        f <mode> <sha256> <relpath>          regular file
        d <mode> - <relpath>/                directory
        l <mode> -> <target> <relpath>       symlink (target is not followed)
        o <mode> - <relpath>                 anything else (fifo, socket, ...)
      Sorted by relpath (byte order), so the output is stable across runs.

  oracle_tools.py normalize --tmp BASE [--sort-blocks]   (stdin -> stdout)
      Replace BASE (and its realpath) with <TMP>. With --sort-blocks, the
      paragraphs that start with "● " (the per-project blocks printed by the Go
      tool's dry-run) are sorted by header, because Go prints them in map
      iteration order, which is random.
"""
import hashlib
import os
import stat
import sys


TMP_VARIANTS = []


def sha256_of(path):
    # file contents are hashed with the temp base replaced by <TMP>, so the pointer file and
    # config.yaml (which hold fixture paths) hash the same on every run
    with open(path, "rb") as fh:
        data = fh.read()
    for variant in TMP_VARIANTS:
        data = data.replace(variant, b"<TMP>")
    return hashlib.sha256(data).hexdigest()


def manifest(root, skip_top, skip_name):
    rows = []
    for dirpath, dirnames, filenames in os.walk(root, followlinks=False):
        rel_dir = os.path.relpath(dirpath, root)
        # prune before descending: entries that must not appear in the manifest
        keep = []
        for name in dirnames:
            full = os.path.join(dirpath, name)
            rel = os.path.normpath(os.path.join(rel_dir, name))
            if rel_dir == "." and name in skip_top:
                continue
            if name in skip_name:
                continue
            if os.path.islink(full):
                # os.walk lists symlinks-to-dirs in dirnames but does not follow them
                target = os.readlink(full)
                mode = stat.S_IMODE(os.lstat(full).st_mode)
                rows.append((rel, "l %03o -> %s %s" % (mode, target, rel)))
                continue
            keep.append(name)
            mode = stat.S_IMODE(os.lstat(full).st_mode)
            rows.append((rel, "d %03o - %s/" % (mode, rel)))
        dirnames[:] = keep
        for name in filenames:
            full = os.path.join(dirpath, name)
            rel = os.path.normpath(os.path.join(rel_dir, name))
            if rel_dir == "." and name in skip_top:
                continue
            if name in skip_name:
                continue
            st = os.lstat(full)
            mode = stat.S_IMODE(st.st_mode)
            if stat.S_ISLNK(st.st_mode):
                rows.append((rel, "l %03o -> %s %s" % (mode, os.readlink(full), rel)))
            elif stat.S_ISREG(st.st_mode):
                rows.append((rel, "f %03o %s %s" % (mode, sha256_of(full), rel)))
            else:
                rows.append((rel, "o %03o - %s" % (mode, rel)))
    # relpath may hold non-UTF-8 bytes; sort on the bytes the filesystem gave us
    rows.sort(key=lambda r: os.fsencode(r[0]))
    out = sys.stdout.buffer
    for _, line in rows:
        out.write(os.fsencode(line) + b"\n")


def normalize(base, sort_blocks):
    data = sys.stdin.buffer.read()
    for variant in {base, os.path.realpath(base)}:
        data = data.replace(os.fsencode(variant), b"<TMP>")
    if sort_blocks:
        data = sort_project_blocks(data)
    sys.stdout.buffer.write(data)


def sort_project_blocks(data):
    # paragraphs are separated by a blank line; project blocks start with "● ".
    # The first block carries the output's leading newline: keep that prefix in place
    # (it belongs to the position, not to the block) while the block contents are reordered.
    marker = "● ".encode()
    paras = data.split(b"\n\n")
    is_block = [p.lstrip(b"\n").startswith(marker) for p in paras]
    if sum(is_block) < 2:
        return data
    first = is_block.index(True)
    prefix = paras[first][: len(paras[first]) - len(paras[first].lstrip(b"\n"))]
    blocks = sorted(p.lstrip(b"\n") for p, b in zip(paras, is_block) if b)
    it = iter(blocks)
    merged = []
    for i, (p, b) in enumerate(zip(paras, is_block)):
        if b:
            blk = next(it)
            merged.append(prefix + blk if i == first else blk)
        else:
            merged.append(p)
    return b"\n\n".join(merged)


def main(argv):
    if len(argv) < 2:
        sys.exit(__doc__)
    cmd = argv[1]
    if cmd == "manifest":
        root = argv[2]
        skip_top, skip_name = set(), set()
        i = 3
        while i < len(argv):
            if argv[i] == "--skip-top":
                skip_top.add(argv[i + 1])
            elif argv[i] == "--skip-name":
                skip_name.add(argv[i + 1])
            elif argv[i] == "--tmp":
                for variant in {argv[i + 1], os.path.realpath(argv[i + 1])}:
                    TMP_VARIANTS.append(os.fsencode(variant))
            else:
                sys.exit("unknown flag " + argv[i])
            i += 2
        manifest(root, skip_top, skip_name)
    elif cmd == "normalize":
        base, sort_blocks, i = None, False, 2
        while i < len(argv):
            if argv[i] == "--tmp":
                base = argv[i + 1]
                i += 2
            elif argv[i] == "--sort-blocks":
                sort_blocks = True
                i += 1
            else:
                sys.exit("unknown flag " + argv[i])
        normalize(base, sort_blocks)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
