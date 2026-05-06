<?php

declare(strict_types=1);

class D
{
    public function __construct(
        public DateTimeImmutable $d,
    ) {}
}

function print_date(?D $d = null): void
{
    $dt = $d?->d;
    if ($dt && $dt->getTimestamp() > 0) {
        echo 'oops';
    }
}
