<?php

declare(strict_types=1);

function probe(): string
{
    return bin2hex([1, 2, 3]);
}
