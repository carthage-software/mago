<?php

declare(strict_types=1);

/** @return non-empty-string */
function first_is_set(string $s): string
{
    if (isset($s[0])) {
        return $s;
    }

    return 'a';
}

/** @return non-falsy-string */
function second_is_set(string $s): string
{
    if (isset($s[1])) {
        return $s;
    }

    return 'a';
}

/** @return non-falsy-string */
function second_from_end_is_set(string $s): string
{
    if (isset($s[-2])) {
        return $s;
    }

    return 'a';
}

/**
 * @param ''|'a'|'ab' $l
 *
 * @return 'ab'
 */
function literal(string $l): string
{
    if (isset($l[1])) {
        return $l;
    }

    return 'ab';
}

function checks_a_later_offset(string $data, bool $close): bool
{
    return isset($data[0]) && ($close || isset($data[5]));
}

/** @param non-empty-string $s */
function later_offset_of_non_empty_string(string $s): bool
{
    return isset($s[3]);
}
