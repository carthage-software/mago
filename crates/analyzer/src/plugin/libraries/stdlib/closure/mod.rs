//! PHP Closure method providers.

mod from_callable;
mod get_current;

pub use from_callable::ClosureFromCallableProvider;
pub use get_current::ClosureGetCurrentProvider;
