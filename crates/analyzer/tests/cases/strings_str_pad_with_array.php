<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:invalid-argument */
    return str_pad([1], 5);
}
