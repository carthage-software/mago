<?php

declare(strict_types=1);

/**
 * @param string $_
 */
function accept_string(string $_): void
{
}

/**
 * @param int $_
 */
function accept_int(int $_): void
{
}

/**
 * @param array<int, string> $n
 */
function test_unset_dynamic_key(array $n): void
{
    if (empty($n)) {
        return;
    }

    while (!empty($n)) {
        unset($n[rand(0, 10)]);
    }

    foreach ($n as $v) {
        accept_string($v);
    }
}

/**
 * @param list<string> $list
 */
function test_unset_list_emptiness(array $list): void
{
    while (!empty($list)) {
        unset($list[0]);
    }

    foreach ($list as $v) {
        accept_string($v);
    }
}

/**
 * @param array<string, int> $map
 * @param string $key
 */
function test_unset_map_emptiness(array $map, string $key): void
{
    if (empty($map)) {
        return;
    }

    while (!empty($map)) {
        unset($map[$key]);
    }

    foreach ($map as $v) {
        accept_int($v);
    }
}
