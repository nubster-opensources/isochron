# Contributing to isochron

isochron is currently in **alpha** (v0.1.x): the core parser and occurrence
engine are shipped; future releases will add extended Quartz field support,
optional serde integration and performance improvements. The repository is
public on [nubster-opensources/isochron](https://github.com/nubster-opensources/isochron)
and contributions are welcome.

## Conventions

isochron follows the Nubster general coding standards documented in
[nubster-docs](https://github.com/nubster-opensources/nubster-docs/tree/main/docs/reference/coding-standards).
In short:

- **Trunk-Based Development**: feature branches `feature/<issue>-<slug>` from
  `main`, never commit directly on `main`.
- **Conventional Commits**: all commit messages follow the
  `type(scope): description` format.
- **Rust style**: `clippy::all` and `clippy::pedantic` set to `deny`, MSRV
  pinned in `rust-toolchain.toml` and `Cargo.toml`.
- **No competitor mentions**: the source code, commit messages, pull requests
  and documentation never name competing tools or services.
- **English everywhere**: rustdoc comments, public types, commit messages,
  issues, pull requests and project documentation are all written in English.

## Local setup

```bash
# Pin the Rust toolchain via rustup
rustup show

# Format the code
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features
```

## Contributor License Agreement

Contributions to this project are governed by the Nubster Contributor License
Agreement, hosted at
[github.com/nubster-opensources/cla](https://github.com/nubster-opensources/cla).

On your first pull request, the CLA Assistant bot will automatically prompt
you to sign the CLA. Once signed, your signature applies to all current and
future contributions to any `nubster-opensources` project.

The CLA is a license grant (not a copyright assignment): you keep the
copyright on your contributions and grant Nubster a broad license to use,
sub-license, and re-license them.

## License

By contributing, you agree that your contributions are dual-licensed under the
[MIT License](./LICENSE-MIT) and the
[Apache License, Version 2.0](./LICENSE-APACHE), at the user's option.

Copyright (c) Nubster.
