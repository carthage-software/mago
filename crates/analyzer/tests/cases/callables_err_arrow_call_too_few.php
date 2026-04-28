<?php

declare(strict_types=1);

$adder_two = fn(int $a, int $b): int => $a + $b;
/** @mago-expect analysis:too-few-arguments */
$adder_two();
