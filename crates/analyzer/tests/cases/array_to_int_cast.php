<?php

declare(strict_types=1);

function arrayCast(array $values): int
{
    return (int) $values; // @mago-expect analysis:array-to-int-cast
}

function integerCast(array $values): int
{
    return (integer) $values; // @mago-expect analysis:array-to-int-cast
}

function emptyCast(): int
{
    return (int) []; // @mago-expect analysis:array-to-int-cast
}

/** @return 1 */
function nonEmptyCast(): int
{
    return (int) [10, 20]; // @mago-expect analysis:array-to-int-cast
}

/** @param list<int> $values */
function listCast(array $values): int
{
    return (int) $values; // @mago-expect analysis:array-to-int-cast
}

/** @param array{count: int} $values */
function shapeCast(array $values): int
{
    return (int) $values; // @mago-expect analysis:array-to-int-cast
}

function unionCast(array|int $values): int
{
    return (int) $values; // @mago-expect analysis:array-to-int-cast
}

function nullableCast(?array $values): int
{
    return (int) $values; // @mago-expect analysis:array-to-int-cast
}

/**
 * @template T of array
 * @param T $values
 */
function genericCast(array $values): int
{
    return (int) $values; // @mago-expect analysis:array-to-int-cast
}

/**
 * @template T of array|int
 * @param T $values
 */
function genericUnionCast(array|int $values): int
{
    return (integer) $values; // @mago-expect analysis:array-to-int-cast
}

function scalarCast(string|float|bool|null $value): int
{
    return (int) $value;
}

function mixedCast(mixed $value): int
{
    return (int) $value;
}

function suppressedCast(array $values): int
{
    // @mago-ignore analysis:array-to-int-cast
    return (int) $values;
}
