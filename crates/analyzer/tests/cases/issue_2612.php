<?php

declare(strict_types=1);

/** @param ArrayObject<int, int|null>&IteratorAggregate<int, int> $values */
function readIteratorView(ArrayObject $values): void
{
    foreach ($values as $value) {
        needsInt($value);
    }

    needsInts(...$values);
}

/** @param IteratorAggregate<int, int>&ArrayObject<int, int|null> $values */
function readReversedIteratorView(ArrayObject $values): void
{
    foreach ($values as $value) {
        needsInt($value);
    }

    needsInts(...$values);
}

/** @param ArrayObject<int|string, int>&IteratorAggregate<string, int> $values */
function readIteratorKeys(ArrayObject $values): void
{
    foreach ($values as $key => $value) {
        needsString($key);
        needsInt($value);
    }
}

/** @param IteratorAggregate<int, int|string>&Traversable<int, int|float> $values */
function readOverlappingUnions(IteratorAggregate $values): void
{
    foreach ($values as $value) {
        needsInt($value);
    }
}

/** @param ArrayObject<int, int|string>&ArrayAccess<int, covariant int> $values */
function readOffsetView(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        needsInt($value);
    }
}

/** @param ArrayAccess<int, covariant int>&ArrayObject<int, int|string> $values */
function readReversedOffsetView(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        needsInt($value);
    }
}

/** @param ArrayAccess<int, int>&ArrayAccess<string, string> $values */
function readDifferentKeys(ArrayAccess $values): void
{
    $integer = $values[0] ?? null;
    if ($integer !== null) {
        needsInt($integer);
        // @mago-expect analysis:invalid-argument
        needsString($integer);
    }

    $string = $values['name'] ?? null;
    if ($string !== null) {
        needsString($string);
        // @mago-expect analysis:invalid-argument
        needsInt($string);
    }
}

/** @param ArrayObject<int|string, int|string>&ArrayAccess<int, covariant int> $values */
function readUnknownKey(ArrayObject $values, int|string $key): void
{
    $value = $values[$key] ?? null;
    if ($value !== null) {
        // @mago-expect analysis:possibly-invalid-argument
        needsInt($value);
    }
}

/**
 * @param ArrayObject<int, int|string>&ArrayAccess<int, covariant int> $values
 * @return ArrayObject<int, int|string>
 */
function preserveMutableType(ArrayObject $values): ArrayObject
{
    $values[0] = 'updated';
    // @mago-expect analysis:invalid-array-access-assignment-value
    $values[0] = true;

    return $values;
}

/** @param ArrayObject<int, int|null> $values */
function unprojectedValues(ArrayObject $values): void
{
    foreach ($values as $value) {
        // @mago-expect analysis:possibly-null-argument
        needsInt($value);
    }
}

function needsInt(int $value): void
{
    echo $value;
}

function needsString(string $value): void
{
    echo $value;
}

function needsInts(int ...$values): void {}
