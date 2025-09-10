<?php

/**
 * @template T
 *
 * @param array<int, T>
 *
 * @return null|list<T>
 */
function as_list_or_nothing(array $arr): null|array
{
    if (array_is_list($arr)) {
        return $arr;
    }

    return null;
}

/**
 * @param array<int, int> $arr
 * @return list<int>
 */
function y(array $arr): array
{
    if (array_is_list($arr)) {
        return $arr;
    }

    return [];
}
