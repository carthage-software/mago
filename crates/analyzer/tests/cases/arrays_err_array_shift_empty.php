<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad_shift(array $xs): int
{
    // @mago-expect analysis:nullable-return-statement
    // @mago-expect analysis:invalid-return-statement
    return array_shift($xs);
}
