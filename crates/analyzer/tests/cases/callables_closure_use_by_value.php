<?php

declare(strict_types=1);

$prefix = 'Hello, ';

$greeter = function (string $name) use ($prefix): string {
    return $prefix . $name;
};

echo $greeter('World');
