<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad_string_index(array $xs): int
{
    /** @mago-expect analysis:mismatched-array-index */
    return $xs['name'];
}
