pub use hashbrown::*;

/// The default hash builder used by the arena-backed hash collections.
pub use hashbrown::DefaultHashBuilder;

/// A hash map whose entries live in an arena.
///
/// The allocator slot is fixed to `&'arena A`, so construct it with
/// [`hashbrown::HashMap::new_in`] (or `with_capacity_in`) and pass `&arena`:
///
/// ```ignore
/// let mut map: HashMap<u32, u32, SharedArena> = HashMap::new_in(&arena);
/// ```
pub type HashMap<'arena, K, V, A, S = DefaultHashBuilder> = hashbrown::HashMap<K, V, S, &'arena A>;

/// A hash set whose elements live in an arena. See [`HashMap`] for construction.
pub type HashSet<'arena, T, A, S = DefaultHashBuilder> = hashbrown::HashSet<T, S, &'arena A>;

/// A low-level hash table whose entries live in an arena. See [`HashMap`] for construction.
pub type HashTable<'arena, T, A> = hashbrown::HashTable<T, &'arena A>;
