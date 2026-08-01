<?php

declare(strict_types=1);

final class Arr
{
    /**
     * @template TKey of array-key
     * @template TValue
     *
     * @param array<TKey, TValue> $array
     * @param callable(TValue, TKey): bool $callback
     * @return array<TKey, TValue>
     */
    public static function where(array $array, callable $callback): array
    {
        return array_filter($array, $callback, ARRAY_FILTER_USE_BOTH);
    }
}

/**
 * @template TKey of array-key
 * @template-covariant TValue
 */
final class Collection
{
    /** @param array<TKey, TValue> $items */
    public function __construct(private array $items) {}

    /**
     * @param (callable(TValue, TKey): bool)|null $callback
     * @return static
     */
    public function filter(?callable $callback = null): static
    {
        if ($callback !== null) {
            return new static(Arr::where($this->items, $callback));
        }

        return new static(array_filter($this->items));
    }
}

/** @param Collection<int, int> $collection */
function takeIntCollection(Collection $collection): void {}

$data = new Collection([10, 20]);
$data = $data->filter(static fn(int $item): bool => $item > 15);

takeIntCollection($data);
