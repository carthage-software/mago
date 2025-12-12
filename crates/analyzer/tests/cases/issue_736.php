<?php

declare(strict_types=1);

final class Picker
{
    /**
     * @param list<T>      $choices
     * @param positive-int $identifier
     *
     * @return T|null
     *
     * @template T
     */
    public static function pickCase(array $choices, int $identifier): mixed
    {
        // @mago-expect analysis:mismatched-array-index
        return $choices[$identifier % count($choices)] ?? null;
    }
}

/**
 * @param list<T>      $choices
 * @param positive-int $identifier
 *
 * @return T|null
 *
 * @template T
 */
function pickCase(array $choices, int $identifier): mixed
{
    // @mago-expect analysis:mismatched-array-index
    return $choices[$identifier % count($choices)] ?? null;
}

enum MyCases: string
{
    case FIRST = 'first';
    case SECOND = 'second';
    case THIRD = 'third';
}

function use_case(null|MyCases $_): void
{
}

use_case(pickCase(MyCases::cases(), identifier: 3));
use_case(Picker::pickCase(MyCases::cases(), identifier: 3));
