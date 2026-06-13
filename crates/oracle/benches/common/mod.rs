//! Shared helpers for the workload benchmark suite.
//!
//! Each bench in `benches/` is one focused workload running a fixed batch of
//! simulated work per iteration. They share:
//!
//! - [`Rng`]: zero-dep deterministic PRNG (xorshift64*) so a given
//!   bench run produces identical inputs across measurements.
//! - [`TypePool`]: a pre-built fixture of representative [`Type`]s
//!   covering the kinds and shapes that turn up in real PHP code, with
//!   a population mix tuned to match what Mago actually feeds in (mostly
//!   singletons, occasional unions, rare deep nesting).
//!
//! There is no global interner in this crate: each bench creates a pair of
//! [`LocalArena`](mago_allocator::LocalArena)s plus a [`TypeBuilder`] in its
//! setup phase and builds the pool against them. The pool cannot own the
//! arenas (that borrow would be self-referential), so it holds only the
//! arena-backed [`Type`]s; benches that need a
//! [`World`](mago_oracle::world::World) construct
//! [`NullWorld`](mago_oracle::world::NullWorld) directly.

use mago_allocator::Arena;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::well_known;

/// Xorshift64* RNG: tiny, deterministic, non-cryptographic. Fine for
/// benches because we want reproducibility, not entropy.
#[derive(Clone, Copy)]
pub struct Rng(u64);

impl Rng {
    /// Seed the generator. A fixed bit is OR-ed in to avoid the all-zero
    /// state.
    pub const fn new(seed: u64) -> Self {
        Self(seed | 0x1)
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Uniform pick from `0..bound` (`bound` must be >= 1).
    #[inline]
    pub fn pick(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound.max(1)
    }

    /// Pick one element from `slice` uniformly. Panics if empty.
    #[inline]
    pub fn pick_from<T>(&mut self, slice: &[T]) -> T
    where
        T: Copy,
    {
        slice[self.pick(slice.len())]
    }
}

/// Wrap a single atom in a (consed) single-atom union.
#[inline]
pub fn ut<'arena, S, A>(builder: &mut TypeBuilder<'_, 'arena, S, A>, atom: Atom<'arena>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    builder.union_of(&[atom])
}

/// A pre-built fixture of representative types. Generated once at bench
/// startup, then drawn from in the hot loop. The mix is roughly
/// 60% singletons (atomic kinds, named objects, literals),
/// 25% small unions (2-4 atoms),
/// 10% wide unions (8-32 atoms),
/// 5%  deeply nested (lists / arrays of unions of …).
pub struct TypePool<'arena> {
    pub singletons: Vec<Type<'arena>>,
    pub small_unions: Vec<Type<'arena>>,
    pub wide_unions: Vec<Type<'arena>>,
    pub deep_nested: Vec<Type<'arena>>,
    pub weighted: Vec<Type<'arena>>,
}

impl<'arena> TypePool<'arena> {
    pub fn new<S, A>(seed: u64, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> TypePool<'arena>
    where
        S: Arena,
        A: Arena,
    {
        let mut rng = Rng::new(seed);
        let singletons = build_singletons(&mut rng, builder);
        let small_unions = build_small_unions(&mut rng, &singletons, builder);
        let wide_unions = build_wide_unions(&mut rng, &singletons, builder);
        let deep_nested = build_deep_nested(&mut rng, &singletons, builder);

        let mut pool =
            TypePool { singletons, small_unions, wide_unions, deep_nested, weighted: Vec::with_capacity(1000) };
        for _ in 0..600 {
            let picked = rng_pick(&mut rng, &pool.singletons);
            pool.weighted.push(picked);
        }
        for _ in 0..250 {
            let picked = rng_pick(&mut rng, &pool.small_unions);
            pool.weighted.push(picked);
        }
        for _ in 0..100 {
            let picked = rng_pick(&mut rng, &pool.wide_unions);
            pool.weighted.push(picked);
        }
        for _ in 0..50 {
            let picked = rng_pick(&mut rng, &pool.deep_nested);
            pool.weighted.push(picked);
        }

        pool
    }
}

fn rng_pick<T>(rng: &mut Rng, slice: &[T]) -> T
where
    T: Copy,
{
    slice[rng.pick(slice.len())]
}

fn build_singletons<'arena, S, A>(rng: &mut Rng, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Vec<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut out: Vec<Type<'arena>> = vec![
        well_known::TYPE_INT,
        well_known::TYPE_STRING,
        well_known::TYPE_FLOAT,
        well_known::TYPE_BOOL,
        well_known::TYPE_NULL,
        well_known::TYPE_VOID,
        well_known::TYPE_MIXED,
        well_known::TYPE_NEVER,
        well_known::TYPE_OBJECT,
        well_known::TYPE_ARRAY_KEY,
        well_known::TYPE_SCALAR,
        well_known::TYPE_NUMERIC,
    ];
    for _ in 0..30 {
        let value = (rng.next_u64() % 200) as i64 - 100;
        out.push(ut(builder, Atom::int_literal(value)));
    }

    let words = ["foo", "bar", "baz", "qux", "hello", "world", "abc", "xyz", "0", "1", ""];
    for _ in 0..30 {
        let word = rng.pick_from(&words);
        let atom = builder.string_literal(word.as_bytes());
        out.push(ut(builder, atom));
    }

    let classes = ["Foo", "Bar", "Baz", "Qux", "Container", "List", "Map", "Set"];
    for class in classes {
        let atom = builder.object_named(class.as_bytes());
        out.push(ut(builder, atom));
    }

    for _ in 0..10 {
        let lower = (rng.next_u64() % 100) as i64;
        let atom = builder.int_range(Some(lower), Some(lower + 50));
        out.push(ut(builder, atom));
    }

    out
}

fn build_small_unions<'arena, S, A>(
    rng: &mut Rng,
    singletons: &[Type<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut out = Vec::with_capacity(40);
    for _ in 0..40 {
        let count = 2 + (rng.next_u32() % 3) as usize;
        let mut atoms: Vec<Atom<'arena>> = Vec::with_capacity(count);
        for _ in 0..count {
            let ty = rng_pick(rng, singletons);
            atoms.extend_from_slice(ty.atoms);
        }
        out.push(builder.union_of(&atoms));
    }

    out
}

fn build_wide_unions<'arena, S, A>(
    rng: &mut Rng,
    singletons: &[Type<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut out = Vec::with_capacity(20);
    for _ in 0..20 {
        let count = 8 + (rng.next_u32() % 25) as usize;
        let mut atoms: Vec<Atom<'arena>> = Vec::with_capacity(count * 2);
        for _ in 0..count {
            let ty = rng_pick(rng, singletons);
            atoms.extend_from_slice(ty.atoms);
        }
        out.push(builder.union_of(&atoms));
    }

    out
}

fn build_deep_nested<'arena, S, A>(
    rng: &mut Rng,
    singletons: &[Type<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut out = Vec::with_capacity(20);
    for _ in 0..15 {
        let inner = rng_pick(rng, singletons);
        let level_one_atom = builder.list_of(inner, false);
        let level_one = ut(builder, level_one_atom);
        let level_two_atom = builder.list_of(level_one, false);
        let level_two = ut(builder, level_two_atom);
        let level_three_atom = builder.list_of(level_two, false);
        out.push(ut(builder, level_three_atom));
    }

    for _ in 0..5 {
        let value_inner = rng_pick(rng, singletons);
        let list_atom = builder.list_of(value_inner, false);
        let list_type = ut(builder, list_atom);
        let keyed_atom = builder.keyed_unsealed(well_known::TYPE_STRING, list_type, false);
        out.push(ut(builder, keyed_atom));
    }

    out
}
