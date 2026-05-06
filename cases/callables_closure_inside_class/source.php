<?php

declare(strict_types=1);

final class Wrapper
{
    /** @return Closure(int): int */
    public function build(): Closure
    {
        return fn(int $n): int => $n * 2;
    }
}

$w = new Wrapper();
$cb = $w->build();
echo $cb(7);
