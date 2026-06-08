//! Common imports for working with arenas.
//!
//! ```
//! use mago_allocator::prelude::*;
//! ```

pub use crate::arena::Arena;
pub use crate::arena::LocalArena;
pub use crate::arena::ScopedArena;
pub use crate::arena::SharedArena;
pub use crate::boxed::Box;
pub use crate::collections::HashMap;
pub use crate::collections::HashSet;
pub use crate::format_in;
pub use crate::iter::CollectIn;
pub use crate::iter::FromIteratorIn;
#[cfg(feature = "rayon")]
pub use crate::iter::ParallelCollectIn;
pub use crate::vec::Vec;
pub use crate::vec_in;
