#!/usr/bin/env python3
"""Check channel documentation consistency.

Rule: any Markdown file mentioning `can0` must also mention either:
- `slcan0`, or
- `can_debugging.md` (central troubleshooting guide link)
"""

from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]

SKIP_DIRS = {
    ".git",
    "target",
    ".venv",
    "releases",
}

CAN0_RE = re.compile(r"\bcan0\b")
SLCAN_RE = re.compile(r"\bslcan0\b")
DEBUG_GUIDE_RE = re.compile(r"can_debugging\.md")


def iter_markdown_files(root: Path):
    for path in root.rglob("*.md"):
        rel_parts = set(path.relative_to(root).parts)
        if rel_parts & SKIP_DIRS:
            continue
        yield path


def main() -> int:
    violations: list[str] = []
    checked = 0

    for md in iter_markdown_files(ROOT):
        text = md.read_text(encoding="utf-8", errors="ignore")
        if not CAN0_RE.search(text):
            continue
        checked += 1
        if SLCAN_RE.search(text) or DEBUG_GUIDE_RE.search(text):
            continue
        violations.append(str(md.relative_to(ROOT)))

    if violations:
        print("[channel-doc-check] FAILED")
        print("Files mention `can0` but neither `slcan0` nor `can_debugging.md` is present:")
        for item in sorted(violations):
            print(f"  - {item}")
        return 1

    print(f"[channel-doc-check] OK: checked {checked} markdown files with can0")
    return 0


if __name__ == "__main__":
    sys.exit(main())
