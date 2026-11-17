<?php

declare(strict_types=1);

function do_while_returns(int $i): string
{
    do {
        return 'a';
    } while ($i > 0);
}

/**
 * @param non-empty-list<int> $values
 */
function foreach_non_empty_returns(array $values): int
{
    foreach ($values as $value) {
        return $value;
    }
}

function foreach_literal_returns(): string
{
    foreach ([1, 2] as $_) {
        return 'a';
    }
}

/**
 * @param list<int> $values
 */
function foreach_maybe_empty_returns(array $values): int
{
    foreach ($values as $value) {
        return $value;
    }

    return 0;
}
