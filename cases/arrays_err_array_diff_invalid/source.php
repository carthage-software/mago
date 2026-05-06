<?php

declare(strict_types=1);

function bad(int $n): array
{
    return array_diff([1, 2, 3], $n);
}
