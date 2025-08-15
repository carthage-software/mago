<?php

/** @psalm-assert-if-true array $v */
function fake_is_array(mixed $v): bool
{
    return fake_is_array($v);
}

/** @psalm-param list<string> $keys */
function &ensure_array_ref(array &$what, array $keys): array
{
    $arr = &$what;
    foreach ($keys as $key) {
        if (!isset($arr[$key]) || !fake_is_array($arr[$key])) {
            $arr[$key] = [];
        }
        $arr = &$arr[$key];
    }

    return $arr;
}

/**
 * @param list<string> $keys
 *
 * @return array<string, array>
 */
function ensure_array_owned(array $arr, array $keys): array
{
    $result = [];
    foreach ($keys as $key) {
        if (!isset($arr[$key]) || !fake_is_array($arr[$key])) {
            $arr[$key] = [];
        }

        $result[$key] = $arr[$key];
    }

    return $result;
}
