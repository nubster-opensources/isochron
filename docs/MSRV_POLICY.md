# Minimum Supported Rust Version (MSRV) policy

The current MSRV is **Rust 1.88** (stable channel).

The MSRV is pinned in `rust-toolchain.toml` at the repository root and
declared in `Cargo.toml` via `rust-version = "1.88"`.

## How the MSRV evolves

- isochron does not commit to supporting Rust versions older than 1.88.
- An MSRV bump is treated as a **minor** version bump per the
  [semver policy](SEMVER_POLICY.md). For example, raising the MSRV from 1.88
  to 1.92 ships in a `0.X.0` release (or `X.0.0` once at 1.0).
- The current MSRV is documented in CHANGELOG.md under the `Changed` section
  of the release that bumps it.

## Why we pick the floor we pick

- **1.88** is required because isochron uses Rust edition 2024 features.
- Future bumps will be driven by concrete features the crate needs, not by
  chasing the latest stable.

## How we verify the MSRV in CI

The CI has two distinct toolchain tracks:

- The `test` job (format, clippy, test suite, doc build) runs on the current
  **stable** toolchain so that the crate always works on the latest stable
  release.
- The dedicated `msrv` job pins **1.88** and runs `cargo test --all-features`,
  which guarantees that no feature requiring a newer compiler has crept in.

## Downstream impact

If you depend on isochron, the dependency resolver will refuse to compile your
project on a Rust version older than the MSRV. You can pin isochron to an
older version only if that version supported your Rust version, as documented
in CHANGELOG.md.
