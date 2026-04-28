<?php

declare(strict_types=1);

function bad(string $s): int|float
{
    // @mago-expect analysis:invalid-argument
    return array_sum($s);
}
