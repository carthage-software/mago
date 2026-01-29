<?php

declare(strict_types=1);

/**
 * @param array<string, array<int, string>> $map
 */
function test_simple_vars(string $key, int $idx, array $map): string
{
    if (isset($map[$key][$idx])) {
        return $map[$key][$idx];
    }
    return '';
}

/**
 * @param array{key: string, idx: int} $item
 * @param array<string, array<int, string>> $map
 */
function test_array_access_keys(array $item, array $map): string
{
    if (isset($map[$item['key']][$item['idx']])) {
        return $map[$item['key']][$item['idx']];
    }
    return '';
}

/**
 * @param array{key: string, idx: int} $item
 * @param array<string, array<int, string>> $map
 */
function test_no_isset(array $item, array $map): string
{
    return $map[$item['key']][$item['idx']];
}

/**
 * @param array{file: string, line: int, function: string} $item
 * @param array<string, array<int, array{file: string, line: int}>> $sourceMap
 */
function f1(array $item, array $sourceMap): string
{
    if (isset($sourceMap[$item['file']][$item['line']])) {
        $item = array_merge($item, $sourceMap[$item['file']][$item['line']]);
    }

    return sprintf('%s at line %s (%s)<br />', $item['file'], $item['line'], $item['function']);
}

/**
 * @param array{file: string, line: int, function: string} $item
 * @param array<string, array<int, array{file: string, line: int}>> $sourceMap
 */
function f2(array $item, array $sourceMap): string
{
    $item = array_merge($item, $sourceMap[$item['file']][$item['line']]);

    return sprintf('%s at line %s (%s)<br />', $item['file'], $item['line'], $item['function']);
}

/**
 * @param array{a: string} $x
 * @param array{b: int} $y
 * @param array{c: string} $z
 * @param array<string, array<int, array<string, bool>>> $map
 */
function test_three_level_nested(array $x, array $y, array $z, array $map): bool
{
    if (isset($map[$x['a']][$y['b']][$z['c']])) {
        return $map[$x['a']][$y['b']][$z['c']];
    }
    return false;
}

/**
 * @param array{idx: int} $item
 * @param array<string, array<int, string>> $map
 */
function test_mixed_keys(string $key, array $item, array $map): string
{
    if (isset($map[$key][$item['idx']])) {
        return $map[$key][$item['idx']];
    }
    return '';
}

/**
 * @param array{key: string} $item
 * @param array<string, array<int, string>> $map
 */
function test_array_then_simple(array $item, int $idx, array $map): string
{
    if (isset($map[$item['key']][$idx])) {
        return $map[$item['key']][$idx];
    }
    return '';
}

/**
 * @param array{inner: array{key: string}} $outer
 * @param array<string, int> $map
 */
function test_deeply_nested_key(array $outer, array $map): int
{
    if (isset($map[$outer['inner']['key']])) {
        return $map[$outer['inner']['key']];
    }
    return 0;
}

/**
 * @param array{k1: string, k2: int} $item
 * @param array<string, string> $map1
 * @param array<int, string> $map2
 */
function test_multiple_isset(array $item, array $map1, array $map2): string
{
    if (isset($map1[$item['k1']]) && isset($map2[$item['k2']])) {
        return $map1[$item['k1']] . $map2[$item['k2']];
    }
    return '';
}

/**
 * @param array{key: string, idx: int} $item
 * @param array<string, array<int, string>> $map
 */
function test_null_coalesce(array $item, array $map): string
{
    return $map[$item['key']][$item['idx']] ?? 'default';
}

/**
 * @param array{idx: int} $item
 * @param array<int, array<int, string>> $map
 */
function test_int_literal_and_array_access(array $item, array $map): string
{
    if (isset($map[0][$item['idx']])) {
        return $map[0][$item['idx']];
    }
    return '';
}

/**
 * @param array{idx: int} $item
 * @param array<string, array<int, string>> $map
 */
function test_string_literal_and_array_access(array $item, array $map): string
{
    if (isset($map['fixed'][$item['idx']])) {
        return $map['fixed'][$item['idx']];
    }
    return '';
}

/**
 * @param array{a: string, b: int} $item
 * @param array<string, array<int, string>> $map
 */
function test_same_var_different_keys(array $item, array $map): string
{
    if (isset($map[$item['a']][$item['b']])) {
        return $map[$item['a']][$item['b']];
    }
    return '';
}

/**
 * @param array{a: string, b: int, c: string, d: int} $k
 * @param array<string, array<int, array<string, array<int, bool>>>> $map
 */
function test_four_levels(array $k, array $map): bool
{
    if (isset($map[$k['a']][$k['b']][$k['c']][$k['d']])) {
        return $map[$k['a']][$k['b']][$k['c']][$k['d']];
    }
    return false;
}
