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
