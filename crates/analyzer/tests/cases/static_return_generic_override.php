<?php

/**
 * @param 'foo'|'bar'|'baz'|'default' $y
 */
function use_string(string $y): void
{
    echo "Using string: $y\n";
}

/**
 * @param 0|1 $x
 */
function use_integer(int $x): void
{
    echo "Using integer: $x\n";
}

/**
 * @template-covariant TKey of array-key
 * @template-covariant TValue
 */
final class Collection
{
    /** @param array<TKey, TValue> $items */
    final public function __construct(
        private $items = [],
    ) {}

    /**
     * @param TKey $key
     *
     * @return TValue|null
     */
    public function get(int|string $key): mixed
    {
        return $this->items[$key] ?? null;
    }

    /**
     * @template TMapValue
     *
     * @param callable(TValue, TKey): TMapValue $callback
     *
     * @return static<TKey, TMapValue>
     */
    public function map(callable $callback): static
    {
        $r = [];
        foreach ($this->items as $key => $value) {
            $r[$key] = $callback($value, $key);
        }

        return new static($r);
    }

    /**
     * @template TReindexKey of array-key
     *
     * @param callable(TValue, TKey): TReindexKey $callback
     *
     * @return static<TReindexKey, TValue>
     */
    public function reindex(callable $callback): static
    {
        $r = [];
        foreach ($this->items as $key => $value) {
            $newKey = $callback($value, $key);
            $r[$newKey] = $value;
        }

        return new static($r);
    }
}

$foo = new Collection([
    'a' => 'foo',
    'b' => 'bar',
    'c' => 'baz',
]);

use_string($foo->get('a') ?? 'default');
use_string($foo->get('b') ?? 'default');
use_string($foo->get('c') ?? 'default');

$bar = $foo->map(
    /**
     * @param 'foo'|'bar'|'baz' $_v
     */
    function (string $_v): int {
        return 1;
    },
)->reindex(
    /**
     * @param 1 $_v
     * @param 'a'|'b'|'c' $k
     */
    function (int $_v, string $k): string {
        return 'key_' . $k;
    },
);

use_integer($bar->get('key_a') ?? 0);
use_integer($bar->get('key_b') ?? 0);
use_integer($bar->get('key_c') ?? 0);
