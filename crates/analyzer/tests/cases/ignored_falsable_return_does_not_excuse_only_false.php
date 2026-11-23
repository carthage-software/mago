<?php

declare(strict_types=1);

/**
 * @return false
 * @ignore-falsable-return
 */
function no(): bool {
    return false;
}

/**
 * @return null
 * @ignore-nullable-return
 */
function nothing(): mixed {
    return null;
}

function f(): int {
    return no();
}

function g(): int {
    return nothing();
}

function h(string $file): int {
    $handle = fopen($file, 'r');
    if (!$handle) {
        return $handle;
    }

    return 1;
}

function k(string $file): int|false
{
    $handle = fopen($file, 'r');
    if (!$handle) {
        return $handle;
    }

    return 1;
}

/** @return resource */
function still_ignored(string $file): mixed
{
    return fopen($file, 'r');
}
