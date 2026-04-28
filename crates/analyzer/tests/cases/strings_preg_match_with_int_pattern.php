<?php

declare(strict_types=1);

function probe(): bool
{
    /** @mago-expect analysis:invalid-argument */
    return (bool) preg_match(42, 'subject');
}
