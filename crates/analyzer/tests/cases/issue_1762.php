<?php

declare(strict_types=1);

namespace test;

/** @return list<int|string> */
function digits(int $num): array
{
    $out = [];

    while ($num > 0) {
        $out[] = $num % 10;
        $num = (int) ($num / 10);
    }

    if (!$out) {
        $out[] = '0';
    }

    return $out;
}

/** @return list<int> */
function negative_threshold(int $count): array
{
    $items = [];

    while ($count >= 5) {
        $items[] = $count;
        $count -= 1;
    }

    if (!$items) {
        return [0];
    }

    return $items;
}
