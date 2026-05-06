<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function describe(array $xs): string
{
    if (count($xs) === 0) {
        return 'empty';
    }
    return (string) $xs[0];
}
