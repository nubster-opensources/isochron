//! isochron: a cron occurrence engine.
//!
//! Parse a Vixie-standard cron expression and compute the next or previous
//! occurrence in strict UTC. Computation is pure and deterministic: there is no
//! scheduling loop, no threads, and no async runtime.
//!
//! ```
//! use isochron::CronError;
//!
//! let err = CronError::EmptyExpression;
//! assert_eq!(err.to_string(), "cron expression must not be empty");
//! ```

mod error;

pub use error::CronError;

// added in a later task
// mod describe;
// mod expression;
// mod field;
// mod iter;
// mod occurrence;
// pub use expression::CronSchedule;
// pub use iter::Upcoming;
