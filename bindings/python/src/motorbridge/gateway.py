from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


def _platform_gateway_name() -> str:
    if sys.platform.startswith("win"):
        return "ws_gateway.exe"
    return "ws_gateway"


def _candidate_paths() -> list[Path]:
    name = _platform_gateway_name()
    here = Path(__file__).resolve()
    candidates: list[Path] = []

    env = os.getenv("MOTORBRIDGE_WS_GATEWAY_BIN")
    if env:
        candidates.append(Path(env).expanduser())

    # Installed wheel layout.
    candidates.append(here.parent / "bin" / name)

    # Source-tree fallback: <repo>/target/release/ws_gateway
    try:
        repo_root = here.parents[4]
        candidates.append(repo_root / "target" / "release" / name)
    except IndexError:
        pass

    # PATH fallback.
    for p in os.getenv("PATH", "").split(os.pathsep):
        if not p:
            continue
        candidates.append(Path(p) / name)

    return candidates


def _resolve_gateway_binary() -> str:
    candidates = _candidate_paths()
    for p in candidates:
        if p.exists():
            return str(p)

    tried = "\n".join(f"- {p}" for p in candidates)
    raise RuntimeError(
        "Cannot locate ws_gateway binary. "
        "Expected packaged path under motorbridge/bin, repo target/release, PATH, "
        "or set MOTORBRIDGE_WS_GATEWAY_BIN.\n"
        f"Tried:\n{tried}"
    )


def run_gateway(argv: list[str] | None = None) -> int:
    gateway_args = list(sys.argv[1:] if argv is None else argv)
    if gateway_args and gateway_args[0] == "--":
        gateway_args = gateway_args[1:]

    exe = _resolve_gateway_binary()
    cmd = [exe, *gateway_args]
    return subprocess.call(cmd)


def main() -> None:
    raise SystemExit(run_gateway())


if __name__ == "__main__":
    main()
