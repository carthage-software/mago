<?php

declare(strict_types=1);

function take(mixed $value): void
{
    echo gettype($value);
}

function run(): void
{
    // @mago-expect analysis:non-existent-class-like
    take(Missing::$count);
    // @mago-expect analysis:non-existent-class-like
    $a = Missing::$count;
    // @mago-expect analysis:non-existent-class-like
    Missing::$count = 1;
    take($a);
}
