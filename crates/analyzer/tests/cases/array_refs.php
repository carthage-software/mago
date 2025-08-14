<?php

/** @psalm-assert-if-true array $v */
function fake_is_array(mixed $v): bool
{
    return fake_is_array($v);
}

/** @psalm-param list<string> $keys */
function &ensure_array(array &$what, array $keys): array
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
