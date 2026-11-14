<?php

declare(strict_types=1);

interface I {}

interface J {}

abstract class B
{
    public const int K = 1;

    public static int $count = 0;

    public int $number = 1;

    public static function s(): int
    {
        return 1;
    }
}

function constant(I $x): int
{
    if ($x instanceof B) {
        return $x::K;
    }

    return 0;
}

function staticProp(I $x): int
{
    if ($x instanceof B) {
        return $x::$count;
    }

    return 0;
}

/** @param I&J&B $x */
function thirdPart(I $x): int
{
    return $x->number;
}

/** @param I&J&B $x */
function thirdStatic(I $x): int
{
    return $x::s();
}
