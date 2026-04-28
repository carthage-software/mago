<?php

declare(strict_types=1);

function probe(float $n): string
{
    return number_format($n, 2);
}
