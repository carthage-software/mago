<?php

declare(strict_types=1);

final class Adder
{
    public static function add(int $a, int $b): int
    {
        return $a + $b;
    }
}

$adder = Adder::add(...);
echo $adder(3, 4);
