<?php

declare(strict_types=1);

/** @param ArrayObject<int, int|null>&IteratorAggregate<int, int> $values */
function readProjectedValues(ArrayObject $values): void
{
    foreach ($values as $value) {
        consumeProjectedInt($value);
    }
    consumeProjectedInts(...$values);
}

/** @param IteratorAggregate<int, int>&ArrayObject<int, int|null> $values */
function readReversedProjection(ArrayObject $values): void
{
    foreach ($values as $value) {
        consumeProjectedInt($value);
    }
}

/** @param ArrayObject<int|string, int>&IteratorAggregate<string, int> $values */
function readProjectedKeys(ArrayObject $values): void
{
    foreach ($values as $key => $value) {
        consumeProjectedString($key);
        consumeProjectedInt($value);
    }
}

/** @param IteratorAggregate<int, int|string>&Traversable<int, int|float> $values */
function readIntersectingValues(IteratorAggregate $values): void
{
    foreach ($values as $value) {
        consumeProjectedInt($value);
    }
}

/** @param ArrayObject<int, int|null> $values */
function readUnprojectedValues(ArrayObject $values): void
{
    foreach ($values as $value) {
        /** @mago-expect analysis:possibly-null-argument */
        consumeProjectedInt($value);
    }
}

/** @template-covariant TState of 'unknown'|'persisted' = 'unknown'|'persisted' */
class ProjectedStateValue
{
    /** @return (TState is 'persisted' ? int : int|null) */
    public function getId(): ?int
    {
        return 1;
    }
}

/** @param ArrayObject<int, ProjectedStateValue>&IteratorAggregate<int, ProjectedStateValue<'persisted'>> $values */
function readProjectedModels(ArrayObject $values): void
{
    foreach ($values as $value) {
        consumeProjectedInt($value->getId());
    }
}

/** @param ArrayObject<int, ProjectedStateValue>&ArrayAccess<int, covariant ProjectedStateValue<'persisted'>|null> $values */
function readProjectedModelOffset(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        consumeProjectedInt($value->getId());
    }
}

function readUnknownModel(ProjectedStateValue $value): void
{
    /** @mago-expect analysis:possibly-null-argument */
    consumeProjectedInt($value->getId());
}

function consumeProjectedInts(int ...$values): void
{
    foreach ($values as $value) {
        echo $value;
    }
}

function consumeProjectedInt(int $value): void
{
    echo $value;
}

function consumeProjectedString(string $value): void
{
    echo $value;
}
