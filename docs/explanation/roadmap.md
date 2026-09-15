# isochron roadmap

isochron is intentionally a small deterministic cron occurrence engine on
[`time`](https://crates.io/crates/time). Its default product boundary is:

- Vixie-style five-field cron, with an optional leading seconds field;
- strict UTC evaluation and no ambient clock;
- occurrence calculation, never job execution or runtime scheduling;
- no default timezone database, Quartz grammar, or asynchronous runtime.

The project should deepen that contract before widening the grammar. Features
that already exist in larger cron engines are not roadmap goals by themselves.
New syntax requires a demonstrated consumer need and must not silently change
the default Vixie semantics.

GitHub milestones are the source of truth for release scope and status. This
document records the sequencing rationale, dependency order, and release gates.
No hard release dates are committed.

## Release sequence

| Version | Theme | Status | Outcome |
|---------|-------|--------|---------|
| [v0.1.0](https://github.com/nubster-opensources/isochron/milestone/1) | Foundation | Shipped | Public deterministic UTC engine |
| [v0.1.1](https://github.com/nubster-opensources/isochron/milestone/3) | Post-release hardening | Shipped | Canonical numeric parsing and safer iteration |
| [v0.1.2](https://github.com/nubster-opensources/isochron/milestone/4) | Trust and correctness | Planned | Correct semantics and a trustworthy release path |
| [v0.2.0](https://github.com/nubster-opensources/isochron/milestone/2) | Semantic API freeze | Planned | Stable value semantics, representation, search, and persistence |
| [v0.3.0](https://github.com/nubster-opensources/isochron/milestone/5) | Portability and iteration | Planned | `no_std`, richer bounded iteration, satisfiability, and performance evidence |
| [v1.0.0](https://github.com/nubster-opensources/isochron/milestone/6) | Stable deterministic core | Planned | Normative grammar and long-term public contract |

Milestones follow the naming format `vX.Y.Z - Theme`, carry no artificial due
date, and contain focused issues with acceptance criteria.

## v0.1.2: trust and correctness

No new product surface should ship before the known correctness and delivery
risks are closed.

- [#19](https://github.com/nubster-opensources/isochron/issues/19):
  pin CI and AI-review implementation code to immutable revisions.
- [#29](https://github.com/nubster-opensources/isochron/issues/29):
  make release preparation and changelog graduation executable end to end.
- [#39](https://github.com/nubster-opensources/isochron/issues/39):
  validate the tag, crate version, and commit before publishing.
- [#40](https://github.com/nubster-opensources/isochron/issues/40):
  make subsecond matching consistent with occurrence search.
- [#42](https://github.com/nubster-opensources/isochron/issues/42):
  extract real changelog sections into GitHub release notes.
- [#43](https://github.com/nubster-opensources/isochron/issues/43):
  keep the quick-start output executable and accurate.
- [#44](https://github.com/nubster-opensources/isochron/issues/44):
  expose day-of-week diagnostics consistent with the accepted grammar.
- [#34](https://github.com/nubster-opensources/isochron/issues/34):
  align governance and policy documentation with repository settings.

Release gate: every P0 issue is closed, the complete release flow has passed a
dry run, and all existing tests, documentation, MSRV, and dependency checks are
green.

## v0.2.0: semantic API freeze

This release may contain deliberate pre-1.0 breaking changes. Its purpose is to
make every observable value and search contract coherent before persistence and
longer-term compatibility depend on them.

Implementation order:

1. [#41](https://github.com/nubster-opensources/isochron/issues/41) fixes the
   semantic equality boundary.
2. [#15](https://github.com/nubster-opensources/isochron/issues/15) defines one
   canonical display form for every semantic schedule.
3. [#17](https://github.com/nubster-opensources/isochron/issues/17) serializes
   only that canonical form and validates every deserialization.
4. [#14](https://github.com/nubster-opensources/isochron/issues/14) adds
   inclusive search after the matching contract from #40 is fixed.
5. [#16](https://github.com/nubster-opensources/isochron/issues/16) replaces the
   implicit fixed search bound with explicit, typed limit semantics.
6. [#45](https://github.com/nubster-opensources/isochron/issues/45) exercises
   the frozen semantics with property, differential, and fuzz testing.
7. [#46](https://github.com/nubster-opensources/isochron/issues/46) makes public
   API changes visible and deliberately classified in CI.

Release gate: equality implies identical canonical display, all serialized
values deserialize through validation, search outcomes are unambiguous, and the
semantic property suite passes.

## v0.3.0: portability and iteration

This release extends where and how the same semantic core can be used. It does
not expand the default cron grammar.

- [#47](https://github.com/nubster-opensources/isochron/issues/47): support
  `no_std + alloc` without changing default behaviour.
- [#48](https://github.com/nubster-opensources/isochron/issues/48): add backward
  and owned occurrence iterators.
- [#49](https://github.com/nubster-opensources/isochron/issues/49): expose lazy,
  bounded occurrence iteration over a time window.
- [#50](https://github.com/nubster-opensources/isochron/issues/50): distinguish
  unsatisfiable schedules from runtime search exhaustion.
- [#51](https://github.com/nubster-opensources/isochron/issues/51): establish
  performance baselines and regression budgets.

Release gate: the standard and no-std builds share the same semantics, every
iterator is bounded or explicitly lazy, and common and worst-case search paths
have measured performance.

## v1.0.0: stable deterministic core

Version 1.0 is a compatibility commitment, not a feature bundle.

- [#52](https://github.com/nubster-opensources/isochron/issues/52) publishes the
  normative UTC Vixie grammar and occurrence specification.
- [#53](https://github.com/nubster-opensources/isochron/issues/53) tracks the
  final compatibility, quality, integration, and release-readiness evidence.

The 1.0 release requires:

- no unresolved P0 or known correctness or supply-chain defect;
- a normative rule-to-test conformance corpus;
- SemVer, MSRV, default-feature, all-feature, and no-std checks;
- stable performance baselines;
- an end-to-end verified publication process;
- validation by Hexeract or another real consumer;
- explicit migration notes for every pre-1.0 breaking change.

## Evidence-gated extensions

Year fields, Quartz operators (`L`, `W`, `#`), per-expression timezones,
localised descriptions, and a CLI are not assigned to a release.

An extension proposal must provide:

1. a concrete consumer and use case;
2. exact syntax and interaction with existing Vixie semantics;
3. an opt-in or parser-mode boundary when compatibility requires it;
4. conformance tests and a migration story;
5. evidence that the feature belongs in this crate rather than an adapter.

Timezone-aware scheduling should normally live in a separate adapter so that
the core remains UTC-only and independent of daylight-saving-time policy.

## Planning conventions

- `priority:P0`: required to ship the target milestone.
- `priority:P1`: high-value milestone scope, expected before release.
- `priority:P2`: planned but deferrable without invalidating the release goal.
- Every open implementation issue belongs to a milestone or is explicitly
  evidence-gated.
- Dependencies are recorded in issue bodies before implementation starts.
- Completed or rejected discovery issues are closed rather than kept as
  permanent speculative backlog.
