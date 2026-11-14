<?php

declare(strict_types=1);

// @mago-expect analysis:missing-return-statement
/** @return Generator<int, int, mixed, string> */
function on_one_path(bool $r): Generator
{
    if ($r) {
        return 'string';
    }

    yield 1;
}

// @mago-expect analysis:missing-return-statement
/** @return Generator<int, int, mixed, string> */
function never_returns(): Generator
{
    yield 1;
}

/** @return Generator<int, int, mixed, string> */
function always_returns(): Generator
{
    yield 1;

    return 'done';
}

/** @return Generator<int, int, mixed, ?string> */
function nullable_return(): Generator
{
    yield 1;
}

/** @return Generator<int, int, mixed, void> */
function void_return(): Generator
{
    yield 1;
}
