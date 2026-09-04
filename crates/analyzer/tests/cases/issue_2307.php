<?php

declare(strict_types=1);

namespace Issue2307;

use function hash_hkdf;
use function strlen;
use function trim;

final class Repro
{
    private const int MIN_LENGTH = 32;

    public static function run(string $input): string
    {
        return self::deriveKey(self::requireValue($input));
    }

    /** @return non-empty-string */
    private static function requireValue(string $value): string
    {
        $value = trim($value);
        if (strlen($value) < self::MIN_LENGTH) {
            exit;
        }

        return $value;
    }

    /** @param non-empty-string $value */
    private static function deriveKey(string $value): string
    {
        return hash_hkdf('sha256', $value, length: 32);
    }
}

/** @return non-empty-string */
function greaterThan(string $value): string
{
    if (strlen($value) > 0) {
        return $value;
    }

    return 'fallback';
}

/** @return non-empty-string */
function greaterThanOrEqual(string $value): string
{
    if (1 <= strlen($value)) {
        return $value;
    }

    return 'fallback';
}

/** @return non-empty-string */
function lessThanOrEqualGuard(string $value): string
{
    if (strlen($value) <= 0) {
        exit;
    }

    return $value;
}

/** @return non-empty-string */
function reversedGreaterThanOrEqualGuard(string $value): string
{
    if (0 >= strlen($value)) {
        exit;
    }

    return $value;
}

/**
 * @param ''|'0' $value
 * @return non-empty-string
 */
function zeroStringRemainsPossible(string $value): string
{
    if (strlen($value) > 0) {
        return $value;
    }

    return 'fallback';
}

/**
 * @mago-expect analysis:invalid-return-statement
 * @return non-empty-string
 */
function upperBoundDoesNotImplyNonEmpty(string $value): string
{
    if (strlen($value) < 32) {
        return $value;
    }

    return 'fallback';
}
