<?php

final class A {}

final class B {}

final class C
{
    public function __construct(
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

/**
 */
function reconcile_literl_class_string(A|B|C $aOrBOrC): C
{
    $name = $aOrBOrC::class;
    if ($name === A::class) {
        // Its A, instantiation is safe
        return C::fromA(new $name());
    }

    if ($name === B::class) {
        // Its B, instantiation is safe
        return C::fromB(new $name());
    }

    if ($name === D::class) {
        // Impossible condition
    }

    return new $name(new B()); // Its C, instantiation is safe
}
