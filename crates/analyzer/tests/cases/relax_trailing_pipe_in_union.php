<?php

/**
 * @param int|string| $x
 */
function takes_trailing(int|string $x): void {}

/**
 * @return iterable<array{0: int|array<string, mixed>|}>
 */
function returns_trailing(): iterable
{
    return [];
}

function test_trailing_pipe(): void
{
    takes_trailing(1);
    takes_trailing('hello');
    foreach (returns_trailing() as $_entry) {
        // nothing
    }
}
