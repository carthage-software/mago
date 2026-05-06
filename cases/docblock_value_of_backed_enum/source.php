<?php

declare(strict_types=1);

enum ColorN: string
{
    case Red = 'red';
    case Green = 'green';
    case Blue = 'blue';
}

/**
 * @return value-of<ColorN>
 */
function any_color(): string
{
    return 'red';
}

echo any_color();
