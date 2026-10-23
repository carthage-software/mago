<?php

declare(strict_types=1);

final class Issue2591Foo
{
    public function __construct(
        public int $prop,
        public ?int $nullableProp = null,
    ) {}
}

function issue_2591_in_if(?Issue2591Foo $value, int $expr): bool
{
    if ($value?->prop === $expr) {
        return true;
    }

    return false;
}

function issue_2591_returned(?Issue2591Foo $value, int $expr): bool
{
    return $value?->prop === $expr;
}

function issue_2591_assigned(?Issue2591Foo $value, int $expr): bool
{
    $x = $value?->prop === $expr;

    return $x;
}

function issue_2591_assigned_not_identical(?Issue2591Foo $value, int $expr): bool
{
    $x = $value?->prop !== $expr;

    return $x;
}

function issue_2591_assigned_nullable_property(?Issue2591Foo $value, int $expr): bool
{
    $x = $value?->nullableProp === $expr;

    return $x;
}

function issue_2591_assigned_impossible_null(int $value, int $expr): bool
{
    // @mago-expect analysis:redundant-comparison,redundant-logical-operation,impossible-null-type-comparison
    $x = $value === null && $expr === 1;

    return $x;
}
