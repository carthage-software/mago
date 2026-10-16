<?php

declare(strict_types=1);

function takes_string(string $_value): void {}

/**
 * @param int|string $a
 * @param float|string $b
 */
function test_union_equality_narrowing(int|string $a, float|string $b): void {
    if ($a === $b) {
        takes_string($a);
        takes_string($b);
    }
}
