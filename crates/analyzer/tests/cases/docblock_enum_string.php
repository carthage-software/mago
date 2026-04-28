<?php

declare(strict_types=1);

enum SuitT: string
{
    case Hearts = 'H';
    case Spades = 'S';
}

/**
 * @param enum-string<SuitT> $cls
 */
function takeEnumStringT(string $cls): void
{
    echo $cls;
}

takeEnumStringT(SuitT::class);
