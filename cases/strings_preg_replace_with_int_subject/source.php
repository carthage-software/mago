<?php

declare(strict_types=1);

function probe(): string|null|array
{
    return preg_replace('/x/', 'y', 42);
}
