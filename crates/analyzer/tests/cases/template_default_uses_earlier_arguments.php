<?php

declare(strict_types=1);

/**
 * @param LimitIterator<int, string> $limit
 * @param CachingIterator<int, string> $caching
 * @param IteratorIterator<int, string> $plain
 */
function spl_wrappers(LimitIterator $limit, CachingIterator $caching, IteratorIterator $plain): void
{
    foreach ($limit as $key => $value) {
        echo $key, $value;
    }

    echo $caching->count(), $plain->valid() ? 'y' : 'n';
}

/**
 * @template K
 * @template V
 * @template I of Iterator<K, V> = Iterator<K, V>
 */
final class Wrapper
{
    /** @param I $inner */
    public function __construct(
        private Iterator $inner,
    ) {}

    /** @return I */
    public function inner(): Iterator
    {
        return $this->inner;
    }
}

/**
 * @param Wrapper<int, string> $w
 *
 * @return Iterator<int, string>
 */
function wrapped(Wrapper $w): Iterator
{
    foreach ($w->inner() as $key => $value) {
        echo $key + 1, strlen($value);
    }

    return $w->inner();
}

/**
 * @extends RecursiveFilterIterator<int, string, RecursiveArrayIterator<int, string>>
 */
final class Filter extends RecursiveFilterIterator
{
    #[Override]
    public function accept(): bool
    {
        return true;
    }
}

function filter_children(Filter $filter): bool
{
    return $filter->getChildren()->accept();
}
