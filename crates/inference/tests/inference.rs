#[path = "inference/harness.rs"]
#[macro_use]
mod harness;

#[path = "inference/extension.rs"]
mod extension;

#[path = "inference/fold/mod.rs"]
mod fold;

#[path = "inference/reconciler.rs"]
mod reconciler;

#[path = "inference/tdd.rs"]
mod tdd;
