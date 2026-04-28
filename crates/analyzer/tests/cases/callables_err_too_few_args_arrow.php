<?php

declare(strict_types=1);

$adder = fn(int $a, int $b): int => $a + $b;
/** @mago-expect analysis:too-few-arguments */
$adder(1);
