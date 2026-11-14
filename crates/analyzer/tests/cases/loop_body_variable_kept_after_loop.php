<?php

declare(strict_types=1);

/**
 * @param list<int> $list
 */
function loop_body_variable_last(array $list): int
{
    foreach ($list as $value) {
        $last = $value;
    }

    return $last ?? 0;
}

/**
 * @param array<int, int> $input
 */
function loop_body_variable_by_reference(array $input): void
{
    foreach ($input as $_) {
        preg_match('/x/', 'x', $match);
    }

    preg_match('/y/', 'y', $match);
}

/**
 * @param list<string> $names
 * @param array<string, string> $cache
 * @throws LogicException
 */
function loop_body_variable_reassigned(array $names, array $cache, string $item, bool $flag): string
{
    if ($flag) {
        foreach ($names as $method) {
            echo $method;
        }
    }

    if (isset($cache[$item])) {
        $method = $cache[$item];
    } elseif ($flag) {
        $method = $item;
    } else {
        throw new LogicException('none');
    }

    return $method;
}
