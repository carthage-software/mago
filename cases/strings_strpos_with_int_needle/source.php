<?php

declare(strict_types=1);

function probe(): int|false
{
    return strpos('hay', 42);
}
