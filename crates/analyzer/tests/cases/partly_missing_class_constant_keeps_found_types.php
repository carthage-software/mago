<?php

declare(strict_types=1);

final class Codes
{
    public const A = 1;
    public const B = 'b';
}

final class OtherCodes
{
    public const A = 2;
}

enum Suit
{
    case Hearts;
}

/**
 * @param 'A'|'Z' $name
 *
 * @return 1
 */
function by_name(string $name): int
{
    // @mago-expect analysis:non-existent-class-constant
    return Codes::{$name};
}

/**
 * @return 'b'
 */
function by_object(Codes|OtherCodes $codes): string
{
    // @mago-expect analysis:non-existent-class-constant
    return $codes::B;
}

/**
 * @param 'Hearts'|'Spades' $name
 */
function by_case(string $name): Suit
{
    // @mago-expect analysis:non-existent-class-constant
    return Suit::{$name};
}

/**
 * @return 1|'b'
 */
function by_ternary(): int|string
{
    return Codes::{mt_rand(0, 1) ? 'A' : 'B'};
}
