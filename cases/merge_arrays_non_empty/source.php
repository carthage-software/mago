<?php

/** @return non-empty-array<int> */
function get_non_empty_array_of_ints(): array
{
    return [1, 2, 3, 'c' => 4];
}

/** @return array<int> */
function get_array_of_ints(): array
{
    return [];
}

/** @return non-empty-list<int> */
function get_non_empty_list_of_ints(): array
{
    return [1, 2, 3];
}

/** @return list<int> */
function get_list_of_ints(): array
{
    return [];
}

/** @param array<int> $arr */
function use_array_of_ints(array $arr): void
{
    foreach ($arr as $v) {
        echo "Value = {$v}\n";
    }
}

/** @param list<int> $arr */
function use_list_of_ints(array $arr): void
{
    foreach ($arr as $v) {
        echo "Value = {$v}\n";
    }
}

/** @param non-empty-array<int> $arr */
function use_non_empty_array_of_ints(array $arr): void
{
    foreach ($arr as $v) {
        echo "Value = {$v}\n";
    }
}

/** @param non-empty-list<int> $arr */
function use_non_empty_list_of_ints(array $arr): void
{
    foreach ($arr as $v) {
        echo "Value = {$v}\n";
    }
}

$array = get_array_of_ints();
$non_empty_array = get_non_empty_array_of_ints();
$list = get_list_of_ints();
$non_empty_list = get_non_empty_list_of_ints();

use_array_of_ints($array);
use_array_of_ints($non_empty_array);
use_array_of_ints($array + $array);
use_array_of_ints(array_merge($array, $array));
use_array_of_ints($array + $non_empty_array);
use_array_of_ints(array_merge($array, $non_empty_array));
use_array_of_ints($non_empty_array + $array);
use_array_of_ints(array_merge($non_empty_array, $array));
use_array_of_ints($non_empty_array + $non_empty_array);
use_array_of_ints(array_merge($non_empty_array, $non_empty_array));

use_non_empty_array_of_ints($non_empty_array);
use_non_empty_array_of_ints($array + $non_empty_array);
use_non_empty_array_of_ints(array_merge($array, $non_empty_array));
use_non_empty_array_of_ints($non_empty_array + $array);
use_non_empty_array_of_ints(array_merge($non_empty_array, $array));
use_non_empty_array_of_ints($non_empty_array + $non_empty_array);
use_non_empty_array_of_ints(array_merge($non_empty_array, $non_empty_array));

use_list_of_ints($list);
use_array_of_ints($list);
use_list_of_ints($non_empty_list);
use_array_of_ints($non_empty_list);
use_list_of_ints($list + $list);
use_list_of_ints(array_merge($list, $list));
use_array_of_ints($list + $list);
use_array_of_ints(array_merge($list, $list));
use_list_of_ints($list + $non_empty_list);
use_list_of_ints(array_merge($list, $non_empty_list));
use_array_of_ints($list + $non_empty_list);
use_array_of_ints(array_merge($list, $non_empty_list));
use_list_of_ints($non_empty_list + $list);
use_list_of_ints(array_merge($non_empty_list, $list));
use_array_of_ints($non_empty_list + $list);
use_array_of_ints(array_merge($non_empty_list, $list));
use_list_of_ints($non_empty_list + $non_empty_list);
use_list_of_ints(array_merge($non_empty_list, $non_empty_list));
use_array_of_ints($non_empty_list + $non_empty_list);
use_array_of_ints(array_merge($non_empty_list, $non_empty_list));

use_non_empty_list_of_ints($non_empty_list);
use_non_empty_array_of_ints($non_empty_list);
use_non_empty_list_of_ints($list + $non_empty_list);
use_non_empty_list_of_ints(array_merge($list, $non_empty_list));
use_non_empty_array_of_ints($list + $non_empty_list);
use_non_empty_array_of_ints(array_merge($list, $non_empty_list));
use_non_empty_list_of_ints($non_empty_list + $list);
use_non_empty_list_of_ints(array_merge($non_empty_list, $list));
use_non_empty_array_of_ints($non_empty_list + $list);
use_non_empty_array_of_ints(array_merge($non_empty_list, $list));
use_non_empty_list_of_ints($non_empty_list + $non_empty_list);
use_non_empty_list_of_ints(array_merge($non_empty_list, $non_empty_list));
use_non_empty_array_of_ints($non_empty_list + $non_empty_list);
use_non_empty_array_of_ints(array_merge($non_empty_list, $non_empty_list));
