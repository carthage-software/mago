<?php

declare(strict_types=1);

function take_int_min(int $x): void
{
    echo $x;
}

function min_empty_returned(): int
{
    /** @mago-expect analysis:never-return */
    return min([]);
}

function min_empty_used(): void
{
    /** @mago-expect analysis:no-value */
    take_int_min(min([]));
}

/**
 * @param non-empty-list<int> $xs
 */
function min_non_empty(array $xs): int
{
    return min($xs);
}
