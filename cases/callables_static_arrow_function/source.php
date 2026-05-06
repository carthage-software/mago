<?php

declare(strict_types=1);

$cube = static fn(int $n): int => $n * $n * $n;

echo $cube(3);
