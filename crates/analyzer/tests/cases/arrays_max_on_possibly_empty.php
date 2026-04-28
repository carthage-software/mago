<?php

declare(strict_types=1);

function take_int_max(int $x): void
{
    echo $x;
}

function max_empty_returned(): int
{
    /** @mago-expect analysis:never-return */
    return max([]);
}

function max_empty_used(): void
{
    /** @mago-expect analysis:no-value */
    take_int_max(max([]));
}

/**
 * @param non-empty-list<int> $xs
 */
function max_non_empty(array $xs): int
{
    return max($xs);
}
