<?php

declare(strict_types=1);

function probe(): int
{
    /** @mago-expect analysis:invalid-argument */
    return strlen([1, 2, 3]);
}
