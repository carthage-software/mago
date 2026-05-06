<?php

declare(strict_types=1);

$counter = 0;

$increment = function () use (&$counter): void {
    $counter += 1;
};

$increment();
$increment();
$increment();

echo $counter;
