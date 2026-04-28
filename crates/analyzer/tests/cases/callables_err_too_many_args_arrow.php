<?php

declare(strict_types=1);

$adder = fn(int $a, int $b): int => $a + $b;
/** @mago-expect analysis:too-many-arguments */
$adder(1, 2, 3);
