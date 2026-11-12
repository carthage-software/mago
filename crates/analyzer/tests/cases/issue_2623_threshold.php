<?php

declare(strict_types=1);

namespace Issue2623Threshold;

/**
 * @param '0'|'1' $left
 * @param '0'|'2' $right
 * @return non-falsy-string
 */
function nonEmptyLiteralUnions(string $left, string $right): string
{
    return $left . $right;
}

/**
 * @param ''|'0' $left
 * @param ''|'0' $right
 * @return non-empty-string
 */
function possiblyEmptyLiteralUnions(string $left, string $right): string
{
    // @mago-expect analysis:invalid-return-statement
    return $left . $right;
}

/**
 * @param ''|'0' $left
 * @param non-empty-string $right
 * @return non-falsy-string
 */
function possiblyEmptyLeft(string $left, string $right): string
{
    // @mago-expect analysis:invalid-return-statement
    return $left . $right;
}

/**
 * @param non-empty-string $left
 * @param ''|'0' $right
 * @return non-falsy-string
 */
function possiblyEmptyRight(string $left, string $right): string
{
    // @mago-expect analysis:invalid-return-statement
    return $left . $right;
}
