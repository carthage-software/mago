use mago_flags::U16Flags;

use crate::ty::Type;
use crate::ty::Typed;
use crate::ty::atom::Atom;
use crate::ty::well_known::NULL;

const fn assert_send_and_sync<T>()
where
    T: Send + Sync,
{
}

const fn coerce_atom<'shorter>(atom: Atom<'static>) -> Atom<'shorter> {
    atom
}

const fn coerce_type<'shorter>(ty: Type<'static>) -> Type<'shorter> {
    ty
}

const fn coerce_typed<'shorter>(typed: Typed<'static>) -> Typed<'shorter> {
    typed
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Atom<'static>>() == 24, "Atom must stay 24 bytes");
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Type<'static>>() == 24, "Type must stay 24 bytes");
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Typed<'static>>() == 32, "Typed must stay 32 bytes");

#[cfg(not(target_pointer_width = "64"))]
const _: () = assert!(size_of::<Atom<'static>>() <= 24, "Atom must not exceed the 64-bit budget");
#[cfg(not(target_pointer_width = "64"))]
const _: () = assert!(size_of::<Type<'static>>() <= 24, "Type must not exceed the 64-bit budget");
#[cfg(not(target_pointer_width = "64"))]
const _: () = assert!(size_of::<Typed<'static>>() <= 32, "Typed must not exceed the 64-bit budget");

const _: () =
    assert!(size_of::<Option<Atom<'static>>>() == size_of::<Atom<'static>>(), "Option<Atom> must use a niche");
const _: () =
    assert!(size_of::<Option<Type<'static>>>() == size_of::<Type<'static>>(), "Option<Type> must use a niche");
const _: () =
    assert!(size_of::<Option<Typed<'static>>>() == size_of::<Typed<'static>>(), "Option<Typed> must use a niche");

const _: () = assert_send_and_sync::<Atom<'static>>();
const _: () = assert_send_and_sync::<Type<'static>>();
const _: () = assert_send_and_sync::<Typed<'static>>();

const TYPE_NULL_VALUE: Type<'static> = Type::from_canonical_atoms(&[NULL]);

const _: Atom<'static> = coerce_atom(NULL);
const _: Type<'static> = coerce_type(TYPE_NULL_VALUE);
const _: Typed<'static> = coerce_typed(Typed { ty: TYPE_NULL_VALUE, flags: U16Flags::empty(), meta: 0 });
