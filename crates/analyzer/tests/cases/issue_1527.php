<?php

declare(strict_types=1);

/** @return false|array{int,int,int,int} */
function helper(): array|false { return false; }

$a = ['total' => 0];

while ($row = helper()) {
    [$count, $y, $m, $d] = $row;

    // This should not report possibly-invalid-operand.
    // $a['total'] is int, $count is int.
    $a['total'] += $count;

    $a[$y] = ['total' => 0];
}
