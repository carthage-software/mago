<?php

declare(strict_types=1);

/**
 * @param list<int> $list
 *
 * @return array<int, int>
 */
function list_negative_key_write(array $list): array
{
    $list[-1] = 1;

    return $list;
}

/**
 * @param list<int> $list
 *
 * @return list<int>
 */
function list_negative_key_write_is_not_a_list(array $list): array
{
    $list[-1] = 1;

    // @mago-expect analysis:invalid-return-statement
    return $list;
}
