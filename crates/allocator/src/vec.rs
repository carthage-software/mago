pub use allocator_api2::vec::*;

/// A contiguous growable array whose elements live in an arena.
///
/// The allocator slot is fixed to `&'arena A`, so construct it with
/// [`new_in`](allocator_api2::vec::Vec::new_in) (or `with_capacity_in`) and pass `&arena`.
pub type Vec<'arena, T, A> = allocator_api2::vec::Vec<T, &'arena A>;
