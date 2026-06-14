//! isochron: a cron occurrence engine.
//!
//! Parse a Vixie-standard cron expression and compute the next or previous
//! occurrence in strict UTC. Computation is pure and deterministic: there is no
//! scheduling loop, no threads, and no async runtime.

// Module declarations and re-exports are enabled task by task as each module
// is implemented. See the implementation plan for the full list.

// added in a later task
// mod describe;
// mod error;
// mod expression;
// mod field;
// mod iter;
// mod occurrence;
// pub use error::CronError;
// pub use expression::CronSchedule;
// pub use iter::Upcoming;
