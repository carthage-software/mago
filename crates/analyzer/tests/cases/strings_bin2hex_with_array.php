<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:invalid-argument */
    return bin2hex([1, 2, 3]);
}
