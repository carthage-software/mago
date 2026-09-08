<?php

declare(strict_types=1);

function probe(string $h): int
{
    /** @mago-expect analysis:possibly-false-operand */
    return strpos($h, 'x') + 1;
}
