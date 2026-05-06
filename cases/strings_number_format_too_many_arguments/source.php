<?php

declare(strict_types=1);

function probe(): string
{
    return number_format(3.14, 2, '.', ',', 'extra');
}
