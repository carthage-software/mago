<?php

declare(strict_types=1);

/**
 * @param list<int> $xs
 */
function bad_string_index(array $xs): int
{
    return $xs['name'];
}
