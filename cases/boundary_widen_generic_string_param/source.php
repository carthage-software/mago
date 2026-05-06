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
    /** @var Bag<string> */
    public Bag $bag;

    public function __construct()
    {
        $this->bag = new Bag('mago');
    }
}

$h = new Holder();
if ($h->bag->value === 'analyzer') {
    echo 'matched';
}
