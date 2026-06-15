# Security policy

## Supported versions

isochron follows the [semver policy](docs/SEMVER_POLICY.md). During the 0.x
phase, only the latest minor release receives security fixes. The current
supported release line is **0.1.x** (MSRV: Rust 1.88).

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

The supported window will be widened once isochron reaches 1.0.

## Reporting a vulnerability

If you find a security vulnerability in isochron, please **do not** open a
public issue. Disclosure rules:

1. Email a detailed report to **security@nubster.com** with the subject prefix
   `[isochron security]`.
2. The report should include:
   - A description of the vulnerability and the attacker model.
   - Affected versions.
   - Reproduction steps or a proof of concept.
   - The impact you anticipate (denial of service, information leak, etc.).
   - Suggested mitigation if you have one.
3. You will receive an acknowledgement within **7 calendar days**. If you do
   not, please follow up at the same address.
4. We will work with you to validate, scope and remediate the issue. A
   coordinated disclosure timeline will be agreed in writing. The default
   embargo period is **90 days** from acknowledgement.
5. Once a fix is published, you will be credited in the release notes unless
   you prefer to remain anonymous.

## Encrypted reporting

If your report includes confidential proof-of-concept material, please encrypt
it with the Nubster security GPG key. The fingerprint and public key are
published at <https://nubster.com/.well-known/security.txt>.

## Out of scope

The following are explicitly **out of scope** for vulnerability reports:

- Issues in unsupported versions.
- Vulnerabilities in third-party dependencies that are already publicly
  disclosed and tracked upstream. Report them to the upstream project.
- Reports based on theoretical attacks without a working proof of concept.

## Public security advisories

Confirmed and fixed vulnerabilities are published on the repository security
advisories page. RustSec advisories are also coordinated for severe issues
when applicable.
