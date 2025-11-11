<?php

declare(strict_types=1);

class test
{
    private const NUMS = ['a' => ['value' => 1], 'b' => ['value' => 2]];

    public function test(string $s): void
    {
        $a = self::NUMS[$s] ?? ['value' => 0];
        $b = self::NUMS[$s]['value'] ?? 0;

        x($a['value'] ?? 0);
        x($b);
    }
}

/** @param 0|1|2 $y */
function x(int $y): void
{
    echo $y;
}
