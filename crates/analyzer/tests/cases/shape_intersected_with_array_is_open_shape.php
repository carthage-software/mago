<?php

declare(strict_types=1);

/**
 * @param array{location: string}&array<string, mixed> $a
 */
function location(array $a): string
{
    return $a['location'];
}

/**
 * @param array<string, mixed>&array{location: string} $a
 */
function location_reversed(array $a): string
{
    return $a['location'];
}
