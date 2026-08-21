<?php

declare(strict_types=1);

/**
 * @param int<0, 255> $r
 * @param int<0, 255> $g
 * @param int<0, 255> $b
 * @return int<0, 255>
 */
function issue_2232_luminance(int $r, int $g, int $b): int
{
    return (int) ((0.299 * $r) + (0.587 * $g) + (0.114 * $b));
}

/**
 * @param int<-10, 20> $value
 * @return int<-40, 20>
 */
function issue_2232_negative_coefficient(int $value): int
{
    return (int) (-2.0 * $value);
}

/**
 * @param int<0, 10> $left
 * @param int<0, 10> $right
 * @return int<-2, 5>
 */
function issue_2232_subtraction(int $left, int $right): int
{
    return (int) ((0.5 * $left) - (0.25 * $right));
}

/**
 * @param int<-5, 5> $value
 * @return int<-2, 2>
 */
function issue_2232_division(int $value): int
{
    return (int) ($value / 2.0);
}

/**
 * @param 9007199254740991 $value
 * @return 27021597764222972
 */
function issue_2232_integer_precision(int $value): int
{
    // @mago-expect analysis:invalid-return-statement
    return (int) ($value * 3);
}
