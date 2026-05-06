<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad_shift(array $xs): int
{
    return array_shift($xs);
}
