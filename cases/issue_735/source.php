<?php

declare(strict_types=1);

/**
 * @param non-empty-list $choices
 */
function countNonEmptyList(array $choices): int
{
    return count($choices);
}

enum MyCases: string
{
    case FIRST = 'first';
    case SECOND = 'second';
    case THIRD = 'third';
}

countNonEmptyList(MyCases::cases());
