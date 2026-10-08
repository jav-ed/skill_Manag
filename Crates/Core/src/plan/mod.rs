//! Deciding what a sync would change, without writing anything.

mod build;
mod error;
pub(crate) mod inspect;
mod model;
mod set;

pub use error::PlanError;
pub use model::{ChangeKind, FileChange, PlanKind, SkillPlan, SourceFile};
pub use set::{Plan, PlanCounts, PlanEntry};

#[cfg(test)]
mod tests;
