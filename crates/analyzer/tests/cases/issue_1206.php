<?php

/**
 * @template Literal of null|bool|int|float|string
 * @param Literal $literal
 * @return Literal
 */
function parse_literal(null|bool|int|float|string $literal, mixed $unknow): null|bool|int|float|string
{
    assert($literal === $unknow);
    return $unknow;
}

/**
 * @template Literal of null|bool|int|float|string
 * @param Literal $literal
 * @return Literal
 */
function parse_literal2(null|bool|int|float|string $literal, mixed $unknow): null|bool|int|float|string
{
    if ($literal === $unknow) {
        return $unknow;
    }

    exit(0);
}

/**
 * @template Literal of null|bool|int|float|string
 * @param Literal $literal
 * @return Literal
 */
function parse_literal3(null|bool|int|float|string $literal, mixed $unknow): null|bool|int|float|string
{
    if ($literal !== $unknow) {
        exit(0);
    }

    return $unknow;
}
