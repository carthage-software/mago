<?php

declare(strict_types=1);

/** @param array<mixed> $ar */
function stat_handler_basic(array $ar): string
{
    $have_com = 0;
    $have_hit = 0;
    // @mago-expect analysis:invalid-type-cast,possibly-undefined-string-array-index,mixed-assignment
    foreach ((array) $ar['types'] as $par) {
        // @mago-expect analysis:mixed-array-access
        if ($par[1]) {
            $have_com = 1;
        }

        // @mago-expect analysis:mixed-array-access
        if ($par[2]) {
            $have_hit = 1;
        }
    }

    $end = '';
    if ($have_com) {
        $end .= '...';
    }

    if ($have_hit) {
        $end .= '...';
    }

    return $end;
}

function flagsWithoutLoop(mixed $par): bool
{
    $first = false;
    $second = false;
    // @mago-expect analysis:mixed-array-access
    if ($par[1]) {
        $first = true;
    }

    // @mago-expect analysis:mixed-array-access
    if ($par[2]) {
        $second = true;
    }

    if ($first) {
        return true;
    }

    return $second;
}

function initiallyTrueFlags(mixed $par): bool
{
    $first = true;
    $second = true;
    // @mago-expect analysis:mixed-array-access
    if ($par[1]) {
        $first = false;
    }

    // @mago-expect analysis:mixed-array-access
    if ($par[2]) {
        $second = false;
    }

    if (!$first) {
        return false;
    }

    return $second;
}

function flagsInWhile(mixed $par, bool $repeat): bool
{
    $first = false;
    $second = false;
    while ($repeat) {
        // @mago-expect analysis:mixed-array-access
        if ($par[1]) {
            $first = true;
        }

        // @mago-expect analysis:mixed-array-access
        if ($par[2]) {
            $second = true;
        }

        $repeat = (bool) rand(0, 1);
    }

    if ($first) {
        return true;
    }

    return $second;
}

function flagsInDoWhile(mixed $par, bool $repeat): bool
{
    $first = false;
    $second = false;
    do {
        // @mago-expect analysis:mixed-array-access
        if ($par[1]) {
            $first = true;
        }

        // @mago-expect analysis:mixed-array-access
        if ($par[2]) {
            $second = true;
        }

        $repeat = (bool) rand(0, 1);
    } while ($repeat);

    if ($first) {
        return true;
    }

    return $second;
}

/** @param list<array{bool, bool}> $items */
function typedFlags(array $items): bool
{
    $first = false;
    $second = false;
    foreach ($items as $par) {
        if ($par[0]) {
            $first = true;
        }

        if ($par[1]) {
            $second = true;
        }
    }

    if ($first) {
        return true;
    }

    return $second;
}

function takeString(string $_value): void {}

function knownGuard(?string $value): void
{
    $found = false;
    if ($value !== null) {
        $found = true;
    }

    if ($found) {
        takeString($value);
    }
}

function unknownGuard(mixed $par, ?string $value): void
{
    $found = false;
    // @mago-expect analysis:mixed-array-access
    if ($par[1]) {
        $found = true;
    }

    if ($found) {
        // @mago-expect analysis:possibly-null-argument
        takeString($value);
    }
}

function partiallyKnownGuard(mixed $par, ?string $value): void
{
    $missing = false;
    // @mago-expect analysis:mixed-array-access,mixed-operand
    if ($par[1] && $value === null) {
        $missing = true;
    }

    if (!$missing) {
        // @mago-expect analysis:possibly-null-argument
        takeString($value);
    }
}

function partiallyKnownTrueGuard(mixed $par, ?string $value): void
{
    $found = false;
    // @mago-expect analysis:mixed-array-access,mixed-operand
    if ($par[1] && $value !== null) {
        $found = true;
    }

    if ($found) {
        takeString($value);
    }
}

function unchangedFlag(mixed $par): void
{
    $found = false;
    // @mago-expect analysis:mixed-array-access
    if ($par[1]) {
        echo 'found';
    }

    // @mago-expect analysis:impossible-condition
    if ($found) {
        echo 'unreachable';
    }
}
