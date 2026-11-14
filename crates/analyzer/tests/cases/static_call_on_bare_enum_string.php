<?php

declare(strict_types=1);

/**
 * @param enum-string $enum
 *
 * @return list<UnitEnum>
 */
function list_cases(string $enum): array
{
    return $enum::cases();
}

/** @param enum-string $enum */
function unknown_method(string $enum): void
{
    // @mago-expect analysis:non-existent-method
    $enum::nope();
}
