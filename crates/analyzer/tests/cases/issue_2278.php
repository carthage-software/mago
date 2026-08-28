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
     * @psalm-assert-if-true =R $left
     * @psalm-assert-if-true =L $right
     */
    public static function f(?Id $left, ?Id $right, int $n): bool
    {
        return $left?->inner === $right?->inner && $n === 5;
    }

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
    public static function isSame(?Id $left, ?Id $right): bool
    {
        return $left?->inner === $right?->inner;
    }
}

/** @param null $_ */
function takeNull(mixed $_): void {}

function f(?Id $a): Id
{
    if (Id::f($a, null, 4)) {
        takeNull($a);

        return new Id(0);
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $a;
}

f(null);

function withoutExactAssertion(?Id $a): Id
{
    if (Id::isSame($a, null)) {
        takeNull($a);

        return new Id(0);
    }

    return $a;
}

withoutExactAssertion(null);
