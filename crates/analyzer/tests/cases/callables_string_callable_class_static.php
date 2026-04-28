<?php

declare(strict_types=1);

final class Static2
{
    public static function run(): int
    {
        return 1;
    }
}

/** @var callable-string $cb */
$cb = 'Static2::run';
$cb();
