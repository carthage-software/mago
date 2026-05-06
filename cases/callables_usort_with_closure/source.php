<?php

declare(strict_types=1);

$nums = [3, 1, 4, 1, 5, 9];
usort($nums, fn(int $a, int $b): int => $a <=> $b);
foreach ($nums as $n) {
    echo $n;
}
