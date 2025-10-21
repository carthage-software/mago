//! Services for running specific analysis tools on PHP code.
//!
//! This module contains the service implementations for each of Mago's analysis tools.
//! Each service encapsulates the logic for running a specific tool (linting, analysis,
//! formatting, or architectural guarding) on a codebase.
//!
//! # Available Services
//!
//! - [`analysis::AnalysisService`]: Static analysis with type checking and control flow
//! - [`format::FormatService`]: Code formatting to ensure consistent style
//! - [`guard::GuardService`]: Architectural rule enforcement
//! - [`lint::LintService`]: Linting for code quality and best practices
//!
//! # Architecture
//!
//! Services use a parallel processing pipeline (defined in the private `pipeline` module)
//! to efficiently process large codebases. The pipeline pattern allows services to:
//!
//! 1. Process files in parallel using thread pools
//! 2. Aggregate results from multiple workers
//! 3. Display progress bars for long-running operations
//!
//! # Usage
//!
//! Services are typically created via the [`Orchestrator`](crate::Orchestrator) factory
//! methods rather than being instantiated directly.

mod pipeline;

pub mod analysis;
pub mod format;
pub mod guard;
pub mod lint;
