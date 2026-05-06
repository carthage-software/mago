<?php

declare(strict_types=1);

/**
 * @param Iterator<string, int> $it
 *
 * @return list<string>
 */
function ok(Iterator $it): array
{
    return array_keys(iterator_to_array($it));
}
