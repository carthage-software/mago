<?php

declare(strict_types=1);

/**
 * @return list<list<int>>
 */
function pairs(): array
{
    return array_chunk([1, 2, 3, 4, 5, 6], 2);
}
