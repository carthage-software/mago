<?php

declare(strict_types=1);

function probe(): int
{
    return mb_strlen(42);
}
