<?php

declare(strict_types=1);

interface Foo {}

interface Bar {}

/**
 * @param array<string, mixed> &$data
 */
function test_by_ref(array &$data): void
{
    $data['key'] = 'value';
}

/**
 * @param string &$name
 * @param int &$count
 */
function test_multiple_by_ref(string &$name, int &$count): void
{
    $name = strtoupper($name);
    $count++;
}

/**
 * @param array<int> &...$arrays Variadic by-reference
 */
function test_variadic_by_ref(array &...$arrays): void
{
    foreach ($arrays as &$arr) {
        $arr[] = 0;
    }
}

/**
 * @param Foo & Bar $intersection This is an intersection type, not by-reference
 */
function test_intersection_type(Foo&Bar $intersection): void
{
    echo get_class($intersection);
}

/**
 * @param array<string, mixed> &$data The data array passed by reference
 */
function test_by_ref_with_description(array &$data): void
{
    $data['updated'] = true;
}
