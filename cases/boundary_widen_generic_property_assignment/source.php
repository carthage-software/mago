<?php

declare(strict_types=1);

/**
 * @template T
 */
final class Bag
{
    /** @param T $value */
    public function __construct(
        public mixed $value,
    ) {}
}

final class Holder
{
    /** @var Bag<int> */
    public Bag $bag;

    public function __construct()
    {
        $this->bag = new Bag(5);
    }
}

$h = new Holder();
if ($h->bag->value === 6) {
    echo 'six';
}
