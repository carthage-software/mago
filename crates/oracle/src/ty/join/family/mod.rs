//! Per-family join rules. Each submodule owns the union-merge
//! algebra for one atom kind (or one cross-kind pair). The
//! orchestration in [`super::compute_with`] gates each rule on
//! its [`super::JoinOptions`] toggle.

pub(super) mod array;
pub(super) mod float;
pub(super) mod int;
pub(super) mod list;
pub(super) mod mixed;
pub(super) mod scalar;
pub(super) mod string;
