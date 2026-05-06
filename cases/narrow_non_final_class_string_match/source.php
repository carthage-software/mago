<?php

final class A {}

final class B {}

class C
{
    final public function __construct(
        public A|B $aOrB,
    ) {}

    public static function fromA(A $a): self
    {
        return new self($a);
    }

    public static function fromB(B $b): self
    {
        return new self($b);
    }
}

final class D {}

function get_c_if(A|B|C $aOrBOrC): C
{
    if ($aOrBOrC instanceof C) {
        return $aOrBOrC;
    }

    if ($aOrBOrC instanceof A) {
        return C::fromA($aOrBOrC);
    }

    return C::fromB($aOrBOrC);
}

function get_c_if_class(A|B|C $aOrBOrC): C
{
    if ($aOrBOrC::class === C::class) {
        return $aOrBOrC;
    }

    if ($aOrBOrC::class === A::class) {
        return C::fromA($aOrBOrC);
    }

    if ($aOrBOrC::class === B::class) {
        return C::fromB($aOrBOrC);
    }

    return $aOrBOrC::fromB(new B()); // still could be `C`
}

function get_c_match(A|B|C $aOrBOrC): C
{
    return match ($aOrBOrC::class) {
        C::class => $aOrBOrC,
        A::class => C::fromA($aOrBOrC),
        B::class => C::fromB($aOrBOrC),
        default => $aOrBOrC, // still could be `C`
    };
}

function get_c_switch(A|B|C $aOrBOrC): C
{
    switch ($aOrBOrC::class) {
        case C::class:
            return $aOrBOrC;
        case A::class:
            return C::fromA($aOrBOrC);
        case B::class:
            return C::fromB($aOrBOrC);
        default:
            return $aOrBOrC; // still could be `C`
    }
}

function y(C $x): C
{
    if ($x::class === C::class) {
        return $x; // `$x` is `C`
    } else {
        return new $x($x->aOrB); // `$x` is still `C`
    }
}
