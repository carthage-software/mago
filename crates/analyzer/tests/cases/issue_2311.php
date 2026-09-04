<?php

declare(strict_types=1);

namespace Issue2311;

function acceptIntOrFloat(int|float $_): void {}

function control(bool $left, ?int $right): void
{
    // @mago-expect analysis:possibly-null-operand
    $left /= $right;

    acceptIntOrFloat($left);
}
