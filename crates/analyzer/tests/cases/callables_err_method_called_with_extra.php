<?php

declare(strict_types=1);

final class Pinger
{
    public function ping(): int
    {
        return 1;
    }
}

$p = new Pinger();
/** @mago-expect analysis:too-many-arguments */
$p->ping(1);
