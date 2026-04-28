<?php

declare(strict_types=1);

final class Multiplier
{
    public function __construct(private int $factor)
    {
    }

    public function apply(int $n): int
    {
        return $n * $this->factor;
    }
}

$m = new Multiplier(5);
$apply = $m->apply(...);
echo $apply(4);
