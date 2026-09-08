<?php

declare(strict_types=1);

function probe(string $h): bool
{
    /** @mago-expect analysis:possibly-false-operand */
    return strpos($h, 'x') > 0;
}
