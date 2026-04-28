<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad(array $xs): array
{
    // @mago-expect analysis:invalid-argument
    return array_map(42, $xs);
}
