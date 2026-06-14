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
//! assert_eq!(schedule.to_string(), "0 0 * * *");
//! ```

mod describe;
mod error;
mod expression;
pub(crate) mod field;
// populated in a later task
// mod iter;
// mod occurrence;

pub use error::CronError;
pub use expression::CronSchedule;
// pub use iter::Upcoming;
