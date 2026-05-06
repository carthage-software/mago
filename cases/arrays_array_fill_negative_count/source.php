<?php

declare(strict_types=1);

/**
 * @return array<array-key, mixed>
 */
function array_fill_negative(): array
{
    return array_fill(0, -1, 'x');
}

/**
 * @return array<array-key, mixed>
 */
function array_fill_zero(): array
{
    return array_fill(0, 0, 'x');
}
