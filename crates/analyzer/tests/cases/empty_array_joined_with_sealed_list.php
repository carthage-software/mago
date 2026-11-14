<?php

declare(strict_types=1);

function from_ternary(bool $flag, int $value): int
{
    $items = $flag ? [] : [$value];

    if (!$items) {
        return 0;
    }

    return $items[0];
}

function from_assignment(bool $flag, int $value): int
{
    $items = [];
    if ($flag) {
        $items = [$value];
    }

    if ($items === []) {
        return 0;
    }

    return $items[0];
}

function unchecked(bool $flag, int $value): int
{
    $items = $flag ? [] : [$value];

    // @mago-expect analysis:possibly-undefined-int-array-index
    // @mago-expect analysis:nullable-return-statement
    // @mago-expect analysis:invalid-return-statement
    return $items[0];
}
