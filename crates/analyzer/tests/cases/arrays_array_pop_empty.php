<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function pop_maybe(array $xs): int
{
    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return array_pop($xs);
}
