<?php

declare(strict_types=1);

namespace Issue2591;

final class Value
{
    public function __construct(
        public int $prop,
        public ?int $nullableProp,
        public ?Value $child = null,
    ) {}
}

function assigned(?Value $value, int $expected): bool
{
    $matches = $value?->prop === $expected;

    return $matches;
}

function compound(?Value $value, int $expected): bool
{
    $matches = $value?->prop === $expected && $value->prop > 0;

    return $matches;
}

function impossiblePropertyNull(?Value $value, int $expected): bool
{
    // @mago-expect analysis:redundant-comparison,redundant-logical-operation,impossible-null-type-comparison
    $matches = $value?->prop === $expected && $value->prop === null;

    return $matches;
}

function returned(?Value $value, int $expected): bool
{
    return $value?->prop === $expected;
}

function condition(?Value $value, int $expected): int
{
    if ($value?->prop === $expected) {
        return $value->prop;
    }

    return 0;
}

function storedGuard(?Value $value, int $expected): int
{
    $matches = $value?->prop === $expected;

    if ($matches) {
        return $value->prop;
    }

    return 0;
}

function nullableProperty(?Value $value, int $expected): int
{
    $matches = $value?->nullableProp === $expected;

    if ($matches) {
        return $value->nullableProp;
    }

    return 0;
}

function reversed(?Value $value, int $expected): int
{
    $matches = $expected === $value?->prop;

    if ($matches) {
        return $value->prop;
    }

    return 0;
}

function notIdentical(?Value $value, int $expected): int
{
    $differs = $value?->prop !== $expected;

    if (!$differs) {
        return $value->prop;
    }

    return 0;
}

function nested(?Value $value, int $expected): int
{
    $matches = $value?->child?->prop === $expected;

    if ($matches) {
        return $value->child->prop;
    }

    return 0;
}

function nullableInequality(?Value $value, int $expected): void
{
    $differs = $value?->prop !== $expected;

    if ($differs) {
        // @mago-expect analysis:possibly-null-argument
        acceptValue($value);
    }
}

function acceptValue(Value $_): void {}

function impossibleNull(int $value, int $expected): bool
{
    // @mago-expect analysis:redundant-comparison,redundant-logical-operation,impossible-null-type-comparison
    $matches = $value === null && $expected === 1;

    return $matches;
}
