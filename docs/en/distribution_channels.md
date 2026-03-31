# Distribution Channels (CI Automation)

This document describes additional package channels built on top of GitHub Releases and PyPI.

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

## 2) Homebrew (Formula in this repo)

Files:

- `Formula/motor-cli.rb`
- `.github/workflows/release-macos-cli.yml`
- `.github/workflows/homebrew-formula-update.yml`

Flow:

1. `release-macos-cli.yml` builds and uploads `motor-cli-<tag>-macos-arm64.tar.gz` to Release.
2. `homebrew-formula-update.yml` downloads that archive, computes SHA256, and updates `Formula/motor-cli.rb`.

Usage example:

```bash
brew tap tianrking/motorbridge
brew install motor-cli
```

## 3) Windows Package Managers Metadata

Workflow: `.github/workflows/windows-package-metadata.yml`

Generator script:

- `tools/release/gen_windows_manifests.py`

Generated files:

- Scoop: `packaging/windows/scoop/motor-cli.json`
- Winget: `packaging/windows/winget/manifests/...`
- Chocolatey template: `packaging/windows/choco/*`

Notes:

- Winget/choco community publishing still requires their upstream submission flows.
- This workflow keeps repository metadata synchronized with each tagged release.
