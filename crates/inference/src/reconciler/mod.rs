use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_oracle::assertion::Assertion;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::float::LiteralFloat;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known::EMPTY_ARRAY;
use mago_oracle::ty::well_known::EMPTY_STRING;
use mago_oracle::ty::well_known::FALSE;
use mago_oracle::ty::well_known::NULL;
use mago_oracle::ty::well_known::TYPE_NULL;
use ordered_float::OrderedFloat;

/// Narrows `ty` under `assertion`, leaning on the oracle lattice.
///
/// An unhandled assertion leaves the type unchanged, and a narrowing with no inhabitants
/// yields `never` (the caller reads that as an impossible/unreachable path).
#[must_use]
pub fn reconcile<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    assertion: Assertion<'arena>,
    ty: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    match assertion {
        Assertion::IsIdentical(atom) | Assertion::IsType(atom) | Assertion::IsEqual(atom) => {
            let narrowing = builder.union_of(&[atom]);

            meet_with(builder, symbols, ty, narrowing)
        }
        Assertion::IsNotIdentical(atom) | Assertion::IsNotType(atom) | Assertion::IsNotEqual(atom) => {
            let removed = builder.union_of(&[atom]);

            subtract_with(builder, symbols, ty, removed)
        }
        Assertion::Truthy => filter_truthy(builder, symbols, ty),
        Assertion::Falsy => filter_falsy(builder, symbols, ty),
        Assertion::IsIsset => subtract_with(builder, symbols, ty, TYPE_NULL),
        Assertion::IsNotIsset => meet_with(builder, symbols, ty, TYPE_NULL),
        _ => ty,
    }
}

/// The lattice meet (intersection) of two types: the values inhabiting both.
#[must_use]
pub fn meet_with<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
    other: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut report = LatticeReport::new();

    meet::compute(ty, other, symbols, LatticeOptions::default(), &mut report, builder)
}

/// `ty` with every value also inhabiting `other` removed.
#[must_use]
pub fn subtract_with<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
    other: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut report = LatticeReport::new();

    subtract::compute(ty, other, symbols, LatticeOptions::default(), &mut report, builder)
}

fn filter_truthy<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let falsy = falsy_type(builder);

    subtract_with(builder, symbols, ty, falsy)
}

fn filter_falsy<'arena, S, A>(
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
    symbols: &SymbolTable<'arena, A>,
    ty: Type<'arena>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let falsy = falsy_type(builder);

    meet_with(builder, symbols, ty, falsy)
}

fn falsy_type<'arena, S, A>(builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let zero_string = builder.intern(b"0");
    let zero_string = builder.string(StringAtom {
        literal: StringLiteral::Value(zero_string),
        casing: StringCasing::Unspecified,
        flags: U8Flags::empty(),
    });

    builder.union_of(&[
        NULL,
        FALSE,
        Atom::Int(IntAtom::Literal(0)),
        Atom::Float(FloatAtom::Literal(LiteralFloat(OrderedFloat(0.0)))),
        EMPTY_STRING,
        zero_string,
        EMPTY_ARRAY,
    ])
}
