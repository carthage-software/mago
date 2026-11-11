<?php

declare(strict_types=1);

namespace Issue2655;

/** @return 5 */
function singleLiteralLength(): int
{
    return strlen('hello');
}

/** @return 0 */
function emptyLiteralLength(): int
{
    return strlen('');
}

/**
 * @param 'hello'|'hi' $value
 * @return 2|5
 */
function literalUnionLength(string $value): int
{
    return strlen($value);
}

/**
 * @param 'hello'|'hi' $value
 * @return 2|5
 */
function namedLiteralUnionLength(string $value): int
{
    return strlen(string: $value);
}

/**
 * @param 'ab'|'cd' $value
 * @return 2
 */
function sameLengthLiterals(string $value): int
{
    return strlen($value);
}

/**
 * @param ''|'hello' $value
 * @return 0|5
 */
function possiblyEmptyLiteralLength(string $value): int
{
    return strlen($value);
}

/** @return 2|4 */
function utf8LiteralLength(bool $flag): int
{
    return strlen($flag ? 'é' : '🦀');
}

/** @return 1|3 */
function binaryLiteralLength(bool $flag): int
{
    return strlen($flag ? "\0" : "a\0b");
}

/** @return non-negative-int */
function unknownLength(string $value): int
{
    return strlen($value);
}

function unknownLengthCanDiffer(string $value): bool
{
    return strlen($value) === 5;
}

/** @param 'hi'|numeric-string $value */
function mixedUnionLengthCanDiffer(string $value): bool
{
    return strlen($value) === 2;
}
