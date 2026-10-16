<?php

declare(strict_types=1);

function maybe_string(): ?string {
    return rand(0, 1) ? "hello" : null;
}

function test_or(): string {
    $foo = maybe_string();
    ($foo !== null) || ($foo = "");

    return $foo;
}

function test_low_or(): string {
    $foo = maybe_string();
    ($foo !== null) or ($foo = "");

    return $foo;
}

function test_and(): string {
    $bar = maybe_string();
    ($bar === null) && ($bar = "default");

    return $bar;
}

function test_low_and(): string {
    $bar = maybe_string();
    ($bar === null) and ($bar = "default");

    return $bar;
}

function takes_string(string $s): string {
    return $s;
}

/**
 * @mago-expect analysis:possibly-null-argument
 */
function test_and_keeps_unassigned_variable(): string {
    $nullable = maybe_string();
    $other = "constant";
    ($nullable === null) && ($other = "assigned");

    return takes_string($nullable);
}

function consume(bool $b): bool {
    return $b;
}

/**
 * @assert string $value
 * @throws InvalidArgumentException
 */
function require_string(?string $value): bool
{
    if ($value === null) {
        throw new InvalidArgumentException();
    }

    return rand(0, 1) === 1;
}

/**
 * @throws InvalidArgumentException
 */
function test_and_preserves_unconditional_assertion(?string $value, bool $flag): string
{
    require_string($value) && consume($flag);

    return $value;
}

/**
 * @throws InvalidArgumentException
 */
function test_or_preserves_unconditional_assertion(?string $value, bool $flag): string
{
    require_string($value) || consume($flag);

    return $value;
}
