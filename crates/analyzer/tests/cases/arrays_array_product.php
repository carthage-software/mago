<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function prod(array $xs): int|float
{
    return array_product($xs);
}
