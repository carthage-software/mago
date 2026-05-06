<?php

declare(strict_types=1);

/**
 * @phpstan-type Pair array{a: int, b: int}
 */
final class ValueHolderM
{
    /**
     * @return value-of<Pair>
     */
    public function any(): int
    {
        return 7;
    }
}

$h = new ValueHolderM();
echo $h->any() + 1;
