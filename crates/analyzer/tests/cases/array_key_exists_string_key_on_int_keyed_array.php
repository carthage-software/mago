<?php

declare(strict_types=1);

/**
 * @param list<int> $list
 * @param array<int, string> $ints
 */
function array_key_exists_string_key_on_int_keys(array $list, array $ints): void
{
    /** @mago-expect analysis:impossible-condition */
    if (array_key_exists('a', $list)) {
        echo 1;
    }

    /** @mago-expect analysis:impossible-condition */
    if (key_exists('a', $list)) {
        echo 2;
    }

    /** @mago-expect analysis:impossible-condition */
    if (array_key_exists('x', $ints)) {
        echo 3;
    }

    if (array_key_exists(0, $list) || array_key_exists('5', $ints)) {
        echo 4;
    }
}
