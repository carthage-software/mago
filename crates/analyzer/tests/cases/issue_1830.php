<?php

declare(strict_types=1);

/**
 * @return array{ 0: numeric-string|null, 1: numeric-string|null }
 */
function parse_gps_coordinates(string $gpsPosition): array
{
    $parsed = explode(',', $gpsPosition);

    // @mago-expect lint:yoda-conditions(2)
    if (
        count($parsed) !== 2
        || $parsed[0] === ''
        || $parsed[1] === ''
        || !is_numeric($parsed[0])
        || !is_numeric($parsed[1])
    ) {
        $parsed = [];
    }

    return [$parsed[0] ?? null, $parsed[1] ?? null];
}
