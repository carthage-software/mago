use crate::arena::Arena;

/// A deep copy of a value into an arena, rebinding every internal reference.
///
/// Implementors copy all of their arena-resident data (byte slices, nested
/// references, slices of children) into the given arena, so the output is
/// self-contained: it stays valid after the arena the input lived in is
/// dropped or reset.
///
/// ```
/// use mago_allocator::prelude::*;
///
/// struct Name<'arena> {
///     value: &'arena [u8],
/// }
///
/// impl CopyInto for Name<'_> {
///     type Output<'arena> = Name<'arena>;
///
///     fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
///     where
///         A: Arena,
///     {
///         Name { value: arena.alloc_slice_copy(self.value) }
///     }
/// }
///
/// let source = LocalArena::new();
/// let target = LocalArena::new();
///
/// let name = Name { value: source.alloc_slice_copy(b"App\\Collection") };
/// let copied = name.copy_into(&target);
/// drop(source);
///
/// assert_eq!(copied.value, b"App\\Collection");
/// ```
pub trait CopyInto {
    type Output<'arena>: 'arena;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena;
}

/// Copies `value` into the arena and returns a reference to the copy.
#[must_use]
#[inline]
pub fn copy_ref_into<'arena, T, A>(value: &T, arena: &'arena A) -> &'arena T::Output<'arena>
where
    T: CopyInto,
    A: Arena,
{
    arena.alloc(value.copy_into(arena))
}

/// Copies every element of `values` into the arena as a new slice.
#[must_use]
#[inline]
pub fn copy_slice_into<'arena, T, A>(values: &[T], arena: &'arena A) -> &'arena [T::Output<'arena>]
where
    T: CopyInto,
    A: Arena,
{
    arena.alloc_slice_fill_iter(values.iter().map(|value| value.copy_into(arena)))
}

macro_rules! trivial_copy_into {
    ($ty:ty) => {
        impl CopyInto for $ty {
            type Output<'arena> = $ty;

            fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
            where
                A: Arena,
            {
                *self
            }
        }
    };
    ($($ty:ty),+) => {
        $(trivial_copy_into!($ty);)+
    };
}

trivial_copy_into!((), bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
