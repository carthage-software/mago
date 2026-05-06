<?php

declare(strict_types=1);

/**
 * @param int<0, max> $b
 * @return int<0, max>
 */
function greaterThanFrom(int $a, int $b): int
{
    if ($a > $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<10, 100> $b
 * @return int<10, max>
 */
function greaterThanRange(int $a, int $b): int
{
    if ($a > $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<min, 100> $b
 * @return int<min, 100>
 */
function lessThanTo(int $a, int $b): int
{
    if ($a < $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<10, 100> $b
 * @return int<min, 100>
 */
function lessThanRange(int $a, int $b): int
{
    if ($a < $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<0, max> $b
 * @return int<0, max>
 */
function greaterThanOrEqualFrom(int $a, int $b): int
{
    if ($a >= $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<10, 100> $b
 * @return int<10, max>
 */
function greaterThanOrEqualRange(int $a, int $b): int
{
    if ($a >= $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<min, 100> $b
 * @return int<min, 100>
 */
function lessThanOrEqualTo(int $a, int $b): int
{
    if ($a <= $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<10, 100> $b
 * @return int<min, 100>
 */
function lessThanOrEqualRange(int $a, int $b): int
{
    if ($a <= $b) {
        return $a;
    }
    return $b;
}

/**
 * @param int<0, 50> $a
 * @param int<25, 75> $b
 * @return int<25, 50>
 */
function rangeIntersectionGreaterThan(int $a, int $b): int
{
    if ($a > $b) {
        return $a;
    }
    return 26;
}

/**
 * @param int<25, 75> $a
 * @param int<0, 50> $b
 * @return int<min, 50>
 */
function rangeIntersectionLessThan(int $a, int $b): int
{
    if ($a < $b) {
        return $a;
    }
    return 49;
}
