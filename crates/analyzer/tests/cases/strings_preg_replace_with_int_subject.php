<?php

declare(strict_types=1);

function probe(): string|null|array
{
    /** @mago-expect analysis:invalid-argument */
    return preg_replace('/x/', 'y', 42);
}
