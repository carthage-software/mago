#![allow(clippy::mut_from_ref)]
#![allow(clippy::unnecessary_safety_comment)]

//! Arena allocation for Mago.
//!
//! Three bump arenas, all implementing the [`Arena`] trait:
//!
//! - [`LocalArena`]: single-threaded, fastest, `Send` but `!Sync`.
//! - [`SharedArena`]: `Send + Sync`, safe to share across threads (for example
//!   behind a `&SharedArena` handed to every worker in a `rayon` pass).
//! - [`ScopedArena`]: a thread-local view into a [`SharedArena`] (via
//!   [`SharedArena::scoped`]) for contention-free, non-escaping scratch.
//!
//! Because all three implement [`Arena`] (and the underlying `Allocator` trait),
//! allocation code is written generically over `A: Arena` and never needs to know
//! which arena it was given:
//!
//! ```ignore
//! fn lower<'arena, A: Arena + ?Sized>(arena: &'arena A, ast: &Ast) -> &'arena Ir<'arena> {
//!     arena.alloc(Ir::from(ast))
//! }
//! ```
//!
//! Allocations live as long as the borrow of the arena they came from, and are
//! reclaimed wholesale when the arena is dropped or reset. For growable
//! collections, use the aliases in [`vec`](mod@vec), [`boxed`], and
//! [`collections`], each built with its `*_in(arena)` constructor; to collect an
//! iterator straight into the arena, use [`CollectIn`] (and `ParallelCollectIn`
//! under the `rayon` feature); for an immutable string copied into the arena, use
//! [`Arena::alloc_str`].
//!
//! Each module also re-exports the items of its underlying crate (`allocator_api2`
//! for [`alloc`](mod@alloc), [`boxed`], [`vec`](mod@vec); `hashbrown` for
//! [`collections`]), so anything not surfaced by the aliases above is still
//! reachable.

pub mod alloc;
pub mod arena;
pub mod boxed;
pub mod collections;
pub mod iter;
pub mod prelude;
pub mod vec;

pub use arena::Arena;
pub use arena::LocalArena;
pub use arena::ScopedArena;
pub use arena::SharedArena;

pub use iter::CollectIn;
pub use iter::FromIteratorIn;
#[cfg(feature = "rayon")]
pub use iter::ParallelCollectIn;

/// Builds an arena-allocated [`Vec`](vec::Vec), analogous to [`std::vec!`].
///
/// The first argument is the arena (or any `&allocator`); the elements follow a
/// `;`:
///
/// ```
/// use mago_allocator::prelude::*;
///
/// let arena = LocalArena::new();
/// let evens = vec_in![&arena; 0, 2, 4];
/// assert_eq!(evens.as_slice(), &[0, 2, 4]);
/// ```
#[macro_export]
macro_rules! vec_in {
    ($arena:expr $(,)?) => {{
        $crate::vec::Vec::new_in($arena)
    }};
    ($arena:expr; $element:expr; $count:expr) => {{
        let mut vector = $crate::vec::Vec::new_in($arena);
        vector.resize($count, $element);
        vector
    }};
    ($arena:expr; $($element:expr),* $(,)?) => {{
        let mut vector = $crate::vec::Vec::new_in($arena);
        $( vector.push($element); )*
        vector
    }};
}

/// Formats its arguments into an arena, returning `&mut str`.
///
/// The macro form of [`Arena::alloc_fmt`], reading like [`format!`] but with no
/// intermediate global allocation:
///
/// ```
/// use mago_allocator::prelude::*;
///
/// let arena = LocalArena::new();
/// let label = format_in!(arena, "{}::{}", "App", "VERSION");
/// assert_eq!(label, "App::VERSION");
/// ```
#[macro_export]
macro_rules! format_in {
    ($arena:expr, $($arg:tt)*) => {{
        use $crate::arena::Arena as _;

        ($arena).alloc_fmt(::core::format_args!($($arg)*))
    }};
}
