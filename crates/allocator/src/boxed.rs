/// A heap allocation living in an arena.
///
/// The allocator slot is fixed to `&'arena A`, so construct it with
/// [`new_in`](allocator_api2::boxed::Box::new_in) and pass `&arena`.
pub type Box<'arena, T, A> = allocator_api2::boxed::Box<T, &'arena A>;
