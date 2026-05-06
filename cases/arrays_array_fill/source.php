<?php

declare(strict_types=1);

/**
 * @return list<string>
 */
function fill_three(): array
{
    return array_fill(0, 3, 'x');
}
