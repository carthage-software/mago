<?php

declare(strict_types=1);

interface A {}

interface B {}

function needsBoth(A&B $value): void {}

function needsBothVariadic(A&B ...$values): void {}

/** @param ArrayObject<int, A>&IteratorAggregate<int, B> $values */
function readInterfaces(ArrayObject $values): void
{
    foreach ($values as $value) {
        needsBoth($value);
    }

    needsBothVariadic(...$values);
}

/** @param IteratorAggregate<int, B>&ArrayObject<int, A> $values */
function readReversedInterfaces(ArrayObject $values): void
{
    foreach ($values as $value) {
        needsBoth($value);
    }

    needsBothVariadic(...$values);
}

/** @param ArrayObject<int, A>&ArrayAccess<int, covariant B> $values */
function readInterfaceOffset(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        needsBoth($value);
    }
}

/** @param ArrayAccess<int, covariant B>&ArrayObject<int, A> $values */
function readReversedInterfaceOffset(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        needsBoth($value);
    }
}

/** @param Iterator<int, int>&Generator<int, int|null> $values */
function primaryIterator(Iterator $values): void
{
    foreach ($values as $value) {
        takesInt($value);
    }
}

/** @param Generator<int, int|null>&Iterator<int, int> $values */
function primaryGenerator(Iterator $values): void
{
    foreach ($values as $value) {
        takesInt($value);
    }
}

/** @implements IteratorAggregate<mixed, mixed> */
final class AlternativeIterator implements IteratorAggregate
{
    /** @return ArrayIterator<int, int>|Generator<int, string> */
    public function getIterator(): ArrayIterator|Generator
    {
        return new ArrayIterator([1]);
    }
}

function iteratorAlternatives(AlternativeIterator $values): void
{
    foreach ($values as $value) {
        // @mago-expect analysis:possibly-invalid-argument
        takesInt($value);
    }

    // @mago-expect analysis:possibly-invalid-argument
    takesInts(...$values);
}

/** @param ArrayObject<int, array{x: int|string}|string>&IteratorAggregate<int, array{x: int|float}|string> $values */
function unresolvedShapeUnion(ArrayObject $values): void
{
    foreach ($values as $value) {
        if (is_array($value)) {
            // @mago-expect analysis:possibly-invalid-argument
            takesString($value['x']);
        }
    }
}

/** @param ArrayObject<int, array{x: int|string}>&ArrayAccess<int, covariant array{x: int|float}> $values */
function unresolvedShapeOffset(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        // @mago-expect analysis:possibly-invalid-argument
        takesString($value['x']);
    }
}

/** @param IteratorAggregate<int, numeric|null>&Traversable<int, array-key|null> $values */
function compositeScalarUnion(IteratorAggregate $values): void
{
    foreach ($values as $value) {
        takesNumericKey($value);
        // @mago-expect analysis:possibly-invalid-argument
        takesNull($value);
    }
}

/** @param ArrayObject<int, A|string|null>&IteratorAggregate<int, B|string|null> $values */
function nullableInterfaceUnion(ArrayObject $values): void
{
    foreach ($values as $value) {
        if (is_object($value)) {
            needsBoth($value);
        } else {
            takesNullableString($value);
        }
    }
}

/** @template-covariant T of bool = bool */
class Result
{
    /** @return (T is true ? int : int|null) */
    public function value(): ?int
    {
        return 1;
    }
}

/** @param ArrayObject<int, Result>&IteratorAggregate<int, Result<true>> $values */
function readGenericObjects(ArrayObject $values): void
{
    foreach ($values as $value) {
        takesInt($value->value());
    }
}

/** @param ArrayObject<int, Result>&ArrayAccess<int, covariant Result<true>> $values */
function readGenericObjectOffset(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        takesInt($value->value());
    }
}

/** @param ArrayObject<int, array<int, numeric>|string>&IteratorAggregate<int, array<int, array-key>|string> $values */
function nestedScalarUnion(ArrayObject $values): void
{
    foreach ($values as $value) {
        if (is_array($value)) {
            foreach ($value as $item) {
                // @mago-expect analysis:possibly-invalid-argument
                takesString($item);
            }
        }
    }
}

function takesInt(int $value): void {}

function takesInts(int ...$values): void {}

function takesString(string $value): void {}

function takesNull(null $value): void {}

/** @param int|numeric-string|null $value */
function takesNumericKey(int|string|null $value): void {}

function takesNullableString(?string $value): void {}
