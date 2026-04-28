<?php

declare(strict_types=1);

function probe(): string
{
    /** @mago-expect analysis:too-many-arguments */
    return number_format(3.14, 2, '.', ',', 'extra');
}
