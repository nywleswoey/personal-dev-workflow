#!/usr/bin/env python3
"""Build a file/entry-point index from spec Traceability sections and flag conflicts.

Run from the repo root:

    python3 scripts/spec_index.py            # check all live specs
    python3 scripts/spec_index.py --spec 1   # show entries for one spec
    python3 scripts/spec_index.py --json     # machine-readable output

Exits 0 if no overlaps, 1 if any live spec conflicts with another.
See specs/README.md § Conflicts.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

SPECS_DIR = Path(__file__).resolve().parent.parent / "specs"
SKIP_FILES = {"README.md", "_template.md"}
BACKTICK = re.compile(r"`([^`]+)`")
# Em-dash (—) used in spec bullets to separate path from symbols.
DASH_SPLIT = re.compile(r"\s+\u2014\s+|\s+--\s+")


def parse_frontmatter(text: str) -> dict:
    """Tiny YAML-ish parser for the fields we use: id, status, supersedes, related."""
    if not text.startswith("---"):
        return {}
    end = text.find("\n---", 3)
    if end == -1:
        return {}
    block = text[3:end].strip().splitlines()
    out: dict = {}
    for raw in block:
        line = raw.split("#", 1)[0].strip()
        if not line or ":" not in line:
            continue
        key, _, value = line.partition(":")
        key = key.strip()
        value = value.strip()
        if value.startswith("[") and value.endswith("]"):
            inner = value[1:-1].strip()
            out[key] = [int(x.strip()) for x in inner.split(",") if x.strip()] if inner else []
        elif key == "id":
            try:
                out[key] = int(value)
            except ValueError:
                out[key] = value
        else:
            out[key] = value
    return out


def parse_traceability(text: str) -> tuple[list[str], list[tuple[str, str]]]:
    """Return (impl_files, entry_points) where entry_points is [(file, symbol), ...]."""
    marker = text.find("## Traceability")
    if marker == -1:
        return [], []
    section = text[marker:].split("\n##", 1)[0]
    files: list[str] = []
    entries: list[tuple[str, str]] = []
    for line in section.splitlines():
        stripped = line.strip()
        if not stripped.startswith("-"):
            continue
        if stripped.lower().startswith("- test file"):
            continue  # tests are per-spec, never conflicting
        if "implementation entry points" in stripped.lower():
            continue
        # Bullet under "Implementation entry points": `path` — `sym1`, `sym2`
        # or just `path` — description text.
        ticks = BACKTICK.findall(stripped)
        if not ticks:
            continue
        path = ticks[0]
        files.append(path)
        parts = DASH_SPLIT.split(stripped, maxsplit=1)
        if len(parts) == 2:
            symbols = BACKTICK.findall(parts[1])
            if symbols:
                for sym in symbols:
                    entries.append((path, sym))
            else:
                # Free-text descriptor (e.g. "tasks table"). Treat the descriptor
                # itself as the entry point so it can still collide.
                desc = parts[1].strip().strip("`")
                entries.append((path, desc))
        else:
            entries.append((path, ""))
    return files, entries


def load_specs() -> list[dict]:
    specs = []
    for path in sorted(SPECS_DIR.glob("*.md")):
        if path.name in SKIP_FILES:
            continue
        text = path.read_text()
        fm = parse_frontmatter(text)
        files, entries = parse_traceability(text)
        specs.append({
            "id": fm.get("id"),
            "path": path.name,
            "status": fm.get("status", "draft"),
            "supersedes": fm.get("supersedes", []) or [],
            "related": fm.get("related", []) or [],
            "files": files,
            "entries": entries,
        })
    return specs


def find_conflicts(specs: list[dict]) -> tuple[dict, dict]:
    live = [s for s in specs if s["status"] != "superseded"]
    suppressed: dict[int, set] = {s["id"]: set(s["supersedes"]) for s in live}

    file_idx: dict[str, list[int]] = defaultdict(list)
    entry_idx: dict[tuple, list[int]] = defaultdict(list)
    for s in live:
        for f in s["files"]:
            file_idx[f].append(s["id"])
        for e in s["entries"]:
            entry_idx[e].append(s["id"])

    def filter_overlap(idx):
        out = {}
        for key, ids in idx.items():
            if len(ids) < 2:
                continue
            kept = []
            for sid in ids:
                # If every *other* claimant is suppressed by sid, sid is alone.
                others = [o for o in ids if o != sid]
                if all(o in suppressed.get(sid, set()) for o in others):
                    continue
                kept.append(sid)
            if len(set(kept)) >= 2:
                out[key] = sorted(set(ids))
        return out

    return filter_overlap(file_idx), filter_overlap(entry_idx)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--json", action="store_true", help="emit machine-readable JSON")
    ap.add_argument("--spec", type=int, help="restrict report to a single spec id")
    args = ap.parse_args()

    specs = load_specs()
    file_conflicts, entry_conflicts = find_conflicts(specs)

    if args.spec is not None:
        specs = [s for s in specs if s["id"] == args.spec]
        file_conflicts = {k: v for k, v in file_conflicts.items() if args.spec in v}
        entry_conflicts = {k: v for k, v in entry_conflicts.items() if args.spec in v}

    if args.json:
        print(json.dumps({
            "specs": specs,
            "file_conflicts": {k: v for k, v in file_conflicts.items()},
            "entry_conflicts": {f"{k[0]}::{k[1]}": v for k, v in entry_conflicts.items()},
        }, indent=2, default=str))
        return 1 if (file_conflicts or entry_conflicts) else 0

    for s in specs:
        marker = "" if s["status"] != "superseded" else " [superseded]"
        print(f"spec {s['id']:>04} {s['path']}{marker} — {len(s['files'])} files, {len(s['entries'])} entry points")
    print()

    if not file_conflicts and not entry_conflicts:
        print("OK — no conflicts.")
        return 0

    print("CONFLICTS DETECTED")
    for f, ids in sorted(file_conflicts.items()):
        print(f"  file  {f}  ← claimed by specs {ids}")
    for (path, sym), ids in sorted(entry_conflicts.items()):
        print(f"  entry {path} :: {sym}  ← claimed by specs {ids}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
