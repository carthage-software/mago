<?php

declare(strict_types=1);

enum Kind: string
{
    case A = 'a';
    case B = 'b';
}

function literal_list(): int
{
    $counts = [];
    foreach (['a', 'b'] as $name) {
        $counts[$name] = 1;
    }

    return $counts['a'];
}

/** @param 'a'|'b' $key */
function literal_list_union_key(string $key): int
{
    $counts = [];
    foreach (['a', 'b'] as $name) {
        $counts[$name] = 1;
    }

    return $counts[$key];
}

function enum_cases(Kind $kind): int
{
    $counts = [];
    foreach (Kind::cases() as $case) {
        $counts[$case->value] = 0;
    }

    return $counts[$kind->value];
}

function first_key(): int
{
    $counts = [];
    foreach (['x', 'y'] as $name) {
        $counts[$name] = 1;
    }

    return $counts[array_key_first($counts)];
}

/** @return array{'a': int, 'b': int} */
function write_after_loop(): array
{
    $name = 'a';
    foreach (['a', 'b'] as $name) {
    }

    $counts = [];
    $counts[$name] = 1;

    // @mago-expect analysis:invalid-return-statement
    return $counts;
}
