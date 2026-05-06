<?php

/**
 * @template Literal of null|bool|int|float|string
 * @param Literal $literal
 * @return Literal
 */
function parse_literal(null|bool|int|float|string $literal, mixed $unknown): null|bool|int|float|string
{
    assert($literal === $unknown);
    return $unknown;
}

/**
 * @template Literal of null|bool|int|float|string
 * @param Literal $literal
 * @return Literal
 */
function parse_literal2(null|bool|int|float|string $literal, mixed $unknown): null|bool|int|float|string
{
    if ($literal === $unknown) {
        return $unknown;
    }

    exit(0);
}

/**
 * @template Literal of null|bool|int|float|string
 * @param Literal $literal
 * @return Literal
 */
function parse_literal3(null|bool|int|float|string $literal, mixed $unknown): null|bool|int|float|string
{
    if ($literal !== $unknown) {
        exit(0);
    }

    return $unknown;
}
