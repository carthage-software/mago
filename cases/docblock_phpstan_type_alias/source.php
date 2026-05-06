<?php

declare(strict_types=1);

/**
 * @phpstan-type Pair array{first: int, second: int}
 */
final class PairFactory
{
    /**
     * @return Pair
     */
    public function make(int $a, int $b): array
    {
        return ['first' => $a, 'second' => $b];
    }

    /**
     * @param Pair $p
     */
    public function sum(array $p): int
    {
        return $p['first'] + $p['second'];
    }
}

$f = new PairFactory();
$p = $f->make(1, 2);
echo $f->sum($p);
