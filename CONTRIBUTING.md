# Contributing to motorbridge

Thank you for your interest in contributing.

## Development Setup

1. Install Rust stable toolchain.
2. Clone the repository.
3. Run:

```bash
cargo check
cargo test
```

## Branching and PRs

- Use short feature branches (`feat/...`, `fix/...`, `docs/...`).
- Open pull requests against `main`.
- Keep PRs focused and small when possible.

## Commit Message Style

Recommended format:

- `feat: ...`
- `fix: ...`
- `docs: ...`
- `refactor: ...`
- `chore: ...`

## Before Submitting

Please ensure:

- `cargo check` passes
- `cargo test` passes
- Documentation is updated if behavior changed
- New behavior has at least basic tests where applicable

## Vendor Integration Rules

- Do not change `motor_core` for vendor-specific protocol details.
- Implement new vendors as separate crates (e.g. `motor_vendor_xxx`).
- Keep ABI backward compatible when possible.
