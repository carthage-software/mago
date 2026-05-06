<?php

/**
 * @template K
 * @template V
 *
 * @param list{K, V} $arr
 *
 * @return list{V, K}
 */
function swap_list(array $arr): array
{
    return swap_array($arr);
}

/**
 * @template K
 * @template V
 *
 * @param array{0: K, 1: V} $arr
 *
 * @return array{0: V, 1: K}
 */
function swap_array(array $arr): array
{
    return swap_list($arr);
}

/**
 * @param list{string, int} $arr
 *
 * @return list{int, string}
 */
function swap_string_int_list(array $arr): array
{
    return swap_array($arr);
}

/**
 * @param array{0: string, 1: int} $arr
 *
 * @return array{0: int, 1: string}
 */
function swap_string_int_array(array $arr): array
{
    return swap_list($arr);
}
