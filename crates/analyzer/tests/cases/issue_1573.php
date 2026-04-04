<?php

declare(strict_types=1);

/**
 * @param list<int> $oid_list
 */
function test(array $oid_list): int
{
    if (!count($oid_list)) {
        exit(1);
    }

    return array_shift($oid_list);
}
