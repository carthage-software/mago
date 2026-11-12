<?php

declare(strict_types=1);

namespace Issue2640;

final class A {}

final class B {}

final class Item {}

final class Other {}

final class Repro
{
    /** @phpstan-assert !A|B $value */
    public static function notAOrB(mixed $value): void {}

    /** @phpstan-assert !B|A $value */
    public static function notBOrA(mixed $value): void {}

    /** @phpstan-assert !A $value */
    public static function notA(mixed $value): void {}

    /**
     * @pure
     * @phpstan-assert !A|B $value
     */
    public static function pureNotAOrB(mixed $value): void {}

    /** @phpstan-assert-if-true !A|B $value */
    public static function isNotAOrB(mixed $value): bool
    {
        return !$value instanceof A && !$value instanceof B;
    }

    /** @phpstan-assert-if-false !A|B $value */
    public static function isAOrB(mixed $value): bool
    {
        return $value instanceof A || $value instanceof B;
    }

    /** @phpstan-assert !null|false $value */
    public static function notNullOrFalse(mixed $value): void {}

    /** @phpstan-assert !int|string $value */
    public static function notIntOrString(mixed $value): void {}

    /** @phpstan-assert A|B $value */
    public static function aOrB(mixed $value): void {}
}

function firstMemberPossible(Item|A $value): Item
{
    Repro::notAOrB($value);

    return $value;
}

function secondMemberPossible(Item|B $value): Item
{
    Repro::notAOrB($value);

    return $value;
}

function reversedFirstMemberPossible(Item|A $value): Item
{
    Repro::notBOrA($value);

    return $value;
}

function reversedSecondMemberPossible(Item|B $value): Item
{
    Repro::notBOrA($value);

    return $value;
}

function allMembersPossible(Item|A|B $value): Item
{
    Repro::notAOrB($value);

    return $value;
}

function noMembersPossible(Item $value): Item
{
    Repro::notAOrB($value);
    Repro::notA($value);

    return $value;
}

function pureAssertion(Item|A $value): Item
{
    Repro::pureNotAOrB($value);

    return $value;
}

function conditionalTrue(Item|A $value): Item
{
    if (Repro::isNotAOrB($value)) {
        return $value;
    }

    return new Item();
}

function conditionalFalse(Item|B $value): Item
{
    if (!Repro::isAOrB($value)) {
        return $value;
    }

    return new Item();
}

function earlyReturn(Item|A $value): Item
{
    if (!Repro::isNotAOrB($value)) {
        return new Item();
    }

    return $value;
}

function nullable(Item|null $value): Item
{
    Repro::notNullOrFalse($value);

    return $value;
}

function scalar(float|int $value): float
{
    Repro::notIntOrString($value);

    return $value;
}

function preserveOtherTypes(Item|A|B|Other $value): Item
{
    Repro::notAOrB($value);

    // @mago-expect analysis:invalid-return-statement
    return $value;
}

function redundantAssertion(Item $value): void
{
    // @mago-expect analysis:redundant-type-comparison
    Repro::pureNotAOrB($value);
}

function impossiblePositiveAssertion(Item $value): void
{
    // @mago-expect analysis:impossible-type-comparison
    Repro::aOrB($value);
}
