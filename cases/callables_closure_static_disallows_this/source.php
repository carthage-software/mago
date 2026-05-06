<?php

declare(strict_types=1);

final class HasState
{
    public int $value = 1;

    /** @return Closure(): int */
    public function getStatic(): Closure
    {
        return static fn(): int => 0;
    }
}

$h = new HasState();
$c = $h->getStatic();
echo $c();
