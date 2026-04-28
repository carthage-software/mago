<?php

declare(strict_types=1);

function probe(): int|false
{
    /** @mago-expect analysis:invalid-argument */
    return strpos('hay', 42);
}
