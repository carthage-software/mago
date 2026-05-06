<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad(array $xs): void
{
    array_walk($xs, 42);
}
