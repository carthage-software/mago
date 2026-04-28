<?php

declare(strict_types=1);

$prefix = 'Item ';
$counter = 0;

$generator = function (string $name) use ($prefix, &$counter): string {
    $counter += 1;
    return $prefix . $name . ' #' . $counter;
};

echo $generator('A');
echo $generator('B');
