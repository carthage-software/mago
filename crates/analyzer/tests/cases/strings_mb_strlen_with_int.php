<?php

declare(strict_types=1);

function probe(): int
{
    /** @mago-expect analysis:invalid-argument */
    return mb_strlen(42);
}
