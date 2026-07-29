<?php

declare(strict_types=1);

// @mago-expect analysis:redundant-comparison,redundant-condition
if ([1] === [1]) {
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if (['a' => 'b'] === ['a' => 'b']) {
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if ([] === []) {
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if ([1, 2, [3]] === [1, 2, [3]]) {
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if (['a' => 1, 'b' => ['c' => 2, 'd' => 3]] === ['a' => 1, 'b' => ['c' => 2, 'd' => 3]]) {
}

$left = [1];
$right = [1];

// @mago-expect analysis:redundant-comparison,redundant-condition
if ($left === $right) {
}

// Key order is significant for strict array identity.
if (['a' => 1, 'b' => 2] === ['b' => 2, 'a' => 1]) {
}

/** @param int $value */
function compare_dynamic_array(int $value): void
{
    if ([$value] === [1]) {
    }
}
