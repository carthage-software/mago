<?php

declare(strict_types=1);

final class TwoArg
{
    public function go(int $n): int
    {
        return $n;
    }
}

$o = new TwoArg();
$cb = [$o, 'go'];
/** @mago-expect analysis:too-many-arguments */
$cb(1, 2);
