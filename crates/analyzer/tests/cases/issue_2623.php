<?php

declare(strict_types=1);

namespace Issue2623;

/**
 * @param non-empty-string $left
 * @param non-empty-string $right
 * @return non-falsy-string
 */
function concatenateNonEmpty(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param numeric-string $left
 * @param non-empty-string $right
 * @return non-falsy-string
 */
function concatenateNumericLeft(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param non-empty-string $left
 * @param numeric-string $right
 * @return non-falsy-string
 */
function concatenateNumericRight(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param numeric-string $left
 * @param numeric-string $right
 * @return non-falsy-string
 */
function concatenateNumericStrings(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param non-empty-string $value
 * @return non-falsy-string
 */
function appendZero(string $value): string
{
    return $value . '0';
}

/**
 * @param non-empty-string $value
 * @return non-falsy-string
 */
function prependZero(string $value): string
{
    return '0' . $value;
}

/**
 * @param non-empty-literal-string $left
 * @param non-empty-literal-string $right
 * @return non-falsy-string
 */
function concatenateLiteralOrigins(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param non-empty-string $left
 * @param non-empty-string $right
 * @return non-falsy-string
 */
function compoundConcatenation(string $left, string $right): string
{
    $left .= $right;

    return $left;
}

/**
 * @param non-empty-string $left
 * @param non-empty-string $right
 * @return non-falsy-string
 */
function concatenateChain(string $left, string $middle, string $right): string
{
    return $left . $middle . $right;
}

/**
 * @param non-falsy-string $left
 * @return non-falsy-string
 */
function leftAlreadyTruthy(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param non-falsy-string $right
 * @return non-falsy-string
 */
function rightAlreadyTruthy(string $left, string $right): string
{
    return $left . $right;
}

/** @return '00' */
function literalZeros(): string
{
    return '0' . '0';
}

/**
 * @param non-empty-string $left
 * @return non-falsy-string
 */
function leftMayBeFalsy(string $left, string $right): string
{
    // @mago-expect analysis:invalid-return-statement
    return $left . $right;
}

/**
 * @param non-empty-string $right
 * @return non-falsy-string
 */
function rightMayBeFalsy(string $left, string $right): string
{
    // @mago-expect analysis:invalid-return-statement
    return $left . $right;
}

/** @return non-empty-string */
function mayStillBeEmpty(string $value, bool $flag): string
{
    // @mago-expect analysis:invalid-return-statement
    return $value . ($flag ? '' : '0');
}

/**
 * @param non-empty-string $suffix
 * @return non-falsy-string
 */
function parenthesizedPrefixMayBeFalsy(string $value, bool $flag, string $suffix): string
{
    // @mago-expect analysis:invalid-return-statement
    return ($value . ($flag ? '' : '0')) . $suffix;
}
