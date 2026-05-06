<?php

declare(strict_types=1);

$multiplier = 3;

$triple = fn(int $n): int => $n * $multiplier;

echo $triple(7);
