<?php

declare(strict_types=1);

final readonly class Id
{
    public function __construct(
        private int $inner,
    ) {}

    /**
     * @template L of self|null
     * @template R of self|null
     *
     * @param L $left
     * @param R $right
     *
     * @psalm-assert-if-true R $left
     * @psalm-assert-if-true L $right
     */
    public static function equals(?Id $left, ?Id $right): bool
    {
        return $left?->inner === $right?->inner;
    }

    /**
     * @template L of self|null
     * @template R of self|null
     *
     * @param L $left
     * @param R $right
     *
     * @psalm-assert-if-true =R $left
     * @psalm-assert-if-true =L $right
     */
    public static function exactlyEquals(?Id $left, ?Id $right): bool
    {
        return $left?->inner === $right?->inner;
    }
}

final readonly class C
{
    public function __construct(
        public Id $id,
    ) {}
}

function f1(Id $a, ?Id $b): C
{
    if (Id::equals($a, $b)) {
        return new C($b);
    }

    return new C(new Id(0));
}

function f2(Id $a, ?C $c): C
{
    if (Id::equals($a, $c?->id)) {
        return $c;
    }

    return new C(new Id(0));
}

function f3(Id $a, ?Id $b): C
{
    if (Id::exactlyEquals($a, $b)) {
        return new C($b);
    }

    return new C(new Id(0));
}

function f4(Id $a, ?C $c): C
{
    if (Id::exactlyEquals($a, $c?->id)) {
        return $c;
    }

    return new C(new Id(0));
}
