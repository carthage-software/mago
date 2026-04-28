<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenDoubleBox
{
    /** @param T $a */
    /** @param T $b */
    public function __construct(public mixed $a, public mixed $b)
    {
    }

    /** @return list<T> */
    public function pair(): array
    {
        return [$this->a, $this->b];
    }
}

$g = new GenDoubleBox(1, 2);
foreach ($g->pair() as $n) {
    echo $n + 1;
}
