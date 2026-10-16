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
