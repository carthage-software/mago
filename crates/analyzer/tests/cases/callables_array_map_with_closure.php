<?php

declare(strict_types=1);

$nums = [1, 2, 3, 4];
$doubled = array_map(fn(int $n): int => $n * 2, $nums);
foreach ($doubled as $d) {
    echo $d;
}
