<?php

declare(strict_types=1);

/**
 * @template V
 */
interface Box
{
    /**
     * @return V
     */
    public function get(): mixed;
}

/**
 * @template V
 *
 * @implements Box<V>
 */
final class Impl implements Box
{
    /**
     * @param V $value
     */
    public function __construct(
        private mixed $value,
    ) {}

    public function get(): mixed
    {
        return $this->value;
    }
}

/**
 * @param Box<string> $b
 */
function unwrap(Box $b): string
{
    if ($b instanceof Impl) {
        return $b->get();
    }

    return '';
}

/**
 * @param Traversable<int, string> $t
 */
function first(Traversable $t): ?string
{
    if ($t instanceof ArrayIterator) {
        return $t[0];
    }

    return null;
}
