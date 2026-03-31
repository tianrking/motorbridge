# Distribution Channels (CI Automation)

This document describes the currently enabled additional package channel built on top of GitHub Releases and PyPI.

## 1) APT Repository (GitHub Pages)

Workflow: `.github/workflows/apt-repo-publish.yml`

- Trigger:
  - on Release `published`
  - or manually (`workflow_dispatch`) with `tag`
- Input assets:
  - `*.deb` from the tagged GitHub Release
- Output:
  - APT repository published to `gh-pages` branch at `/apt`
  - URL pattern:
    - `https://<owner>.github.io/<repo>/apt`

Optional signing (recommended in production):

- `APT_GPG_PRIVATE_KEY` (ASCII-armored private key)
- `APT_GPG_PASSPHRASE` (if your key is passphrase-protected)

If no key is configured, workflow publishes unsigned metadata (`-skip-signing`).

## Package Audience Mapping

| Package / Asset | Intended Developer Type | Typical Scenario |
|---|---|---|
| `motorbridge-abi-<tag>-linux-x86_64.deb` | C/C++ robotics developers on Ubuntu/Debian | Install ABI (`libmotor_abi`) + headers + CMake config for native integration |
| `motorbridge-abi-<tag>-linux-*.tar.gz` / `windows-*.zip` | C/C++ developers on non-deb targets / custom toolchains | Manual unpack and link in embedded or cross-compilation environments |
| `motorbridge-*.whl` / `motorbridge-*.tar.gz` | Python application and tooling developers | Build robot apps, calibration tools, factory scripts with Python SDK |
| `motor-cli-<tag>-<platform>.tar.gz/.zip` | Test/ops/factory engineers | Direct motor scan/control without embedding SDK code |

At this stage, Homebrew and Windows package-manager metadata automation were intentionally removed to keep the release process fully deterministic and fully automated in CI.
