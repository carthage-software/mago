<?php

declare(strict_types=1);

/** @param positive-int $x */
function example(int $x): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return $x > 0;
}
