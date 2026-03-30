#!/usr/bin/env python3
import os
import subprocess
import sys


def main() -> int:
    repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    target = os.path.join(repo_root, "examples", "python", "four_vendor_pos_sync.py")
    print(
        "[info] moved to examples/python/four_vendor_pos_sync.py, forwarding...",
        file=sys.stderr,
    )
    return subprocess.call([sys.executable, target, *sys.argv[1:]])


if __name__ == "__main__":
    raise SystemExit(main())
