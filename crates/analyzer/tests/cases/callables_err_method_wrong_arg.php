<?php

declare(strict_types=1);

final class Receiver
{
    public function take(int $n): int
    {
        return $n;
    }
}

$r = new Receiver();
/** @mago-expect analysis:invalid-argument */
$r->take('wrong');
