<?php

declare(strict_types=1);

function bad(int $a): array
{
    return array_merge([1, 2], $a);
}
