<?php

declare(strict_types=1);

function bad(int $n): array
{
    return array_combine($n, ['a', 'b']);
}
