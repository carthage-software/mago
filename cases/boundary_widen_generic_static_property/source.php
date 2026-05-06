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

final class Registry
{
    /** @var Bag<int> */
    public static Bag $bag;
}

Registry::$bag = new Bag(7);
if (Registry::$bag->value === 9) {
    echo 'nine';
}
