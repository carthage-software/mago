<?php

declare(strict_types=1);

function bad(int $n): array
{
    return iterator_to_array($n);
}
