#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/setup_pcbusb_macos.sh --system
#   scripts/setup_pcbusb_macos.sh --user-local
#   scripts/setup_pcbusb_macos.sh --user-local --prefix "$HOME/.local"
#   scripts/setup_pcbusb_macos.sh --archive /path/to/macOS_Library_for_PCANUSB_v0.13.tar.gz --system

usage() {
  cat <<'USAGE'
Install PCBUSB runtime for macOS PCAN support.

Options:
  --system        Install with package install.sh (requires sudo, installs to /usr/local)
  --user-local    Install to user prefix (default: $HOME/.local), no sudo
  --prefix PATH   Prefix for --user-local (default: $HOME/.local)
  --archive PATH  Path to macOS_Library_for_PCANUSB_v0.13.tar.gz
  -h, --help      Show this help

Notes:
  - If neither --system nor --user-local is given, --user-local is used.
  - After --user-local install, export DYLD_LIBRARY_PATH=<prefix>/lib when running motor_cli.
USAGE
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: this script only supports macOS (Darwin)." >&2
  exit 1
fi

mode="user"
prefix="${HOME}/.local"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
archive="${repo_root}/third_party/pcan/macos/macOS_Library_for_PCANUSB_v0.13.tar.gz"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --system)
      mode="system"
      shift
      ;;
    --user-local)
      mode="user"
      shift
      ;;
    --prefix)
      prefix="${2:-}"
      if [[ -z "$prefix" ]]; then
        echo "error: --prefix requires a path" >&2
        exit 1
      fi
      shift 2
      ;;
    --archive)
      archive="${2:-}"
      if [[ -z "$archive" ]]; then
        echo "error: --archive requires a path" >&2
        exit 1
      fi
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ! -f "$archive" ]]; then
  echo "error: archive not found: $archive" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

echo "[pcan] archive: $archive"
tar -xzf "$archive" -C "$tmpdir"

pkg_dir=""
if [[ -d "$tmpdir/PCBUSB" ]]; then
  pkg_dir="$tmpdir/PCBUSB"
else
  maybe_dir="$(find "$tmpdir" -maxdepth 2 -type d -name PCBUSB | head -n1 || true)"
  if [[ -n "$maybe_dir" ]]; then
    pkg_dir="$maybe_dir"
  fi
fi

if [[ -z "$pkg_dir" || ! -d "$pkg_dir" ]]; then
  echo "error: could not locate PCBUSB directory after extraction" >&2
  exit 1
fi

if [[ "$mode" == "system" ]]; then
  if [[ ! -x "$pkg_dir/install.sh" ]]; then
    echo "error: install.sh not found in package" >&2
    exit 1
  fi
  echo "[pcan] running system install via sudo"
  sudo "$pkg_dir/install.sh"
  echo "[pcan] done: system install to /usr/local/lib and /usr/local/include"
  exit 0
fi

mkdir -p "$prefix/lib" "$prefix/include"

dylib_versioned="$(find "$pkg_dir" -maxdepth 1 -type f -name 'libPCBUSB.*.dylib' | sort -V | tail -n1 || true)"
if [[ -z "$dylib_versioned" ]]; then
  echo "error: no libPCBUSB.*.dylib found in package" >&2
  exit 1
fi

cp -f "$dylib_versioned" "$prefix/lib/"
dylib_base="$(basename "$dylib_versioned")"
ln -sf "$prefix/lib/$dylib_base" "$prefix/lib/libPCBUSB.dylib"

if [[ ! -f "$pkg_dir/PCBUSB.h" ]]; then
  echo "error: PCBUSB.h not found in package" >&2
  exit 1
fi
cp -f "$pkg_dir/PCBUSB.h" "$prefix/include/PCBUSB.h"

echo "[pcan] done: user-local install complete"
echo "[pcan] library: $prefix/lib/$dylib_base"
echo "[pcan] symlink: $prefix/lib/libPCBUSB.dylib"
echo "[pcan] header:  $prefix/include/PCBUSB.h"
echo ""
echo "Run motor_cli with:"
echo "  DYLD_LIBRARY_PATH=$prefix/lib ./target/release/motor_cli ..."
