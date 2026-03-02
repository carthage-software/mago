<?php

/**
 * @param array{
 *   left: array{value: int},
 *   right: array{value: int}
 * } $data
 */
function issue1242_reconciler_snapshot(array $data, int $threshold): void
{
    $left = &$data['left']['value'];
    $right = &$data['right']['value'];
    $left += 1;
    $right += 1;

    if (count($data) > $threshold) {
        $total = $left + $right;
        echo (string) $total;
    }
}
