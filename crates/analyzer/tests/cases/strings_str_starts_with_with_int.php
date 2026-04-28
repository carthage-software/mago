<?php

declare(strict_types=1);

function probe(): bool
{
    /** @mago-expect analysis:invalid-argument */
    return str_starts_with(42, 'foo');
}
