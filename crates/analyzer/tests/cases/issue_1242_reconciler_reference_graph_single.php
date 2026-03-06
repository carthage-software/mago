<?php

/**
 * @param array{node: array{value: int}} $data
 */
function issue1242_reconciler_single(array $data, int $threshold): void
{
    $leaf = &$data['node']['value'];
    $leaf += 1;

    if (count($data) > $threshold) {
        echo (string) $leaf;
    }
}
