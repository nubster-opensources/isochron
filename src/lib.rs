//! isochron: a cron occurrence engine.
//!
//! Parse a Vixie-standard cron expression and compute the next or previous
//! occurrence in strict UTC. Computation is pure and deterministic: there is no
//! scheduling loop, no threads, and no async runtime.
//!
//! ```
//! use isochron::CronSchedule;
//! use time::macros::datetime;
//!
//! let schedule = CronSchedule::parse("0 0 * * *").expect("valid expression");
//! let after = datetime!(2026-01-01 12:00:00 UTC);
//! let next = schedule.next_after(after).expect("an occurrence exists");
//! assert_eq!(next, datetime!(2026-01-02 00:00:00 UTC));
//! ```

// Enables the `doc_cfg` attribute on docs.rs, where `--cfg docsrs` is set, so a reader
// sees which items a feature gates instead of believing them unconditional.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod day_filter;
mod describe;
mod error;
mod expression;
pub(crate) mod field;
mod iter;
mod occurrence;
#[cfg(feature = "serde")]
mod serialization;

pub use error::CronError;
pub use expression::CronSchedule;
pub use iter::Upcoming;
pub use occurrence::SEARCH_HORIZON_YEARS;

/// Runs the Rust code blocks of `README.md` as doctests, so the quick start
/// cannot drift from the crate's actual behaviour.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
