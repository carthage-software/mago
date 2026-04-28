<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 * @template C
 */
final class GenTriple
{
    /**
     * @param A $a
     * @param B $b
     * @param C $c
     */
    public function __construct(public mixed $a, public mixed $b, public mixed $c)
    {
    }

    /** @return A */
    public function getA(): mixed
    {
        return $this->a;
    }

    /** @return B */
    public function getB(): mixed
    {
        return $this->b;
    }

    /** @return C */
    public function getC(): mixed
    {
        return $this->c;
    }
}

function takes_tri(int $a, string $b, bool $c): void
{
}

$t = new GenTriple(1, 'two', true);
takes_tri($t->getA(), $t->getB(), $t->getC());
