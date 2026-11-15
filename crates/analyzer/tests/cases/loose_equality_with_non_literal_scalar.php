<?php

declare(strict_types=1);

function int_not_equal_float(int $page, float $f): int
{
    if ($page != $f) {
        return 1;
    }

    return $page;
}

function int_equal_float(int $page, float $f): int
{
    if ($page == $f) {
        return $page;
    }

    return 1;
}

function float_equal_int(float $value, int $i): float
{
    if ($value == $i) {
        return $value;
    }

    return 1.0;
}

function string_equal_float(string $value, float $f): string
{
    if ($value == $f) {
        return $value;
    }

    return '';
}

function int_equal_bool(int $value, bool $b): int
{
    if ($value == $b) {
        return $value;
    }

    return 1;
}

function bool_equal_int(bool $value, int $i): bool
{
    if ($value == $i) {
        return $value;
    }

    return false;
}

function bool_equal_float(bool $value, float $f): bool
{
    if ($value == $f) {
        return $value;
    }

    return false;
}

function bool_equal_string(bool $value, string $s): bool
{
    if ($value == $s) {
        return $value;
    }

    return false;
}

function nullable_int_equal_int(?int $value, int $i): ?int
{
    // @mago-expect analysis:possibly-null-operand
    if ($value == $i) {
        return $value;
    }

    return 1;
}

function pager_resolve_page(string $tmp): int
{
    if (!is_numeric($tmp)) {
        return 1;
    }

    $page = (int) $tmp;
    $f = (float) $tmp;
    if ($page != $f) {
        return 1;
    }

    if ($page < 1 || $page > 99999) {
        return 1;
    }

    return $page;
}
