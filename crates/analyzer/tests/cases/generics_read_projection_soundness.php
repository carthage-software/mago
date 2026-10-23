<?php

declare(strict_types=1);

/** @param ArrayObject<int, array{x:int|string}>&IteratorAggregate<int, array{x:int|float}> $values */
function readShapeProjection(ArrayObject $values): void
{
    foreach ($values as $value) {
        /** @mago-expect analysis:possibly-invalid-argument */
        requireProjectionString($value['x']);
    }
}

/** @param ArrayObject<int, array{x:int|string}>&ArrayAccess<int, covariant array{x:int|float}> $values */
function readOffsetShapeProjection(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        /** @mago-expect analysis:possibly-invalid-argument */
        requireProjectionString($value['x']);
    }
}

/** @param ArrayObject<int, array{x:int|string}|string>&IteratorAggregate<int, array{x:int|float}|string> $values */
function readShapeUnionProjection(ArrayObject $values): void
{
    foreach ($values as $value) {
        if (is_array($value)) {
            /** @mago-expect analysis:possibly-invalid-argument */
            requireProjectionString($value['x']);
        }
    }
}

/** @implements IteratorAggregate<mixed, mixed> */
class AlternativeProjectionIterator implements IteratorAggregate
{
    /** @return ArrayIterator<int, int>|Generator<int, string> */
    public function getIterator(): ArrayIterator|Generator
    {
        return new ArrayIterator([1]);
    }
}

function unpackAlternativeProjection(AlternativeProjectionIterator $values): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    requireProjectionInts(...$values);
}

/** @param Iterator<int, int>&Generator<int, int|null> $values */
function readPrimaryIterator(Iterator $values): void
{
    foreach ($values as $value) {
        requireProjectionInts($value);
    }
}

/** @param Generator<int, int|null>&Iterator<int, int> $values */
function readPrimaryGenerator(Iterator $values): void
{
    foreach ($values as $value) {
        requireProjectionInts($value);
    }
}

function requireProjectionInts(int ...$values): void
{
    foreach ($values as $value) {
        echo $value;
    }
}

function requireProjectionString(string $value): void
{
    echo $value;
}
