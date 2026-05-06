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
$p->ping(1);
