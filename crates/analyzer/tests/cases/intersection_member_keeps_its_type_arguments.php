<?php

declare(strict_types=1);

interface A
{
}

/**
 * @template T
 */
interface B
{
    /** @var T */
    public mixed $bar { get; }

    /**
     * @return T
     */
    public function get(): mixed;
}

/**
 * @param A&B<int> $x
 */
function property(A&B $x): int
{
    return $x->bar;
}

/**
 * @param A&B<int> $x
 */
function method(A&B $x): int
{
    return $x->get();
}
interface C
{
}

interface P
{
    public int $plain { get; }
}

/**
 * @param A&C&P $x
 */
function docblock($x): int
{
    return $x->plain;
}

interface Fluent
{
    public function again(): static;
}

function fluent(A&Fluent $x): A&Fluent
{
    return $x->again();
}
