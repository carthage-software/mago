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
 * @template TKey of array-key
 * @template TValue
 */
final class Collection
{
    /** @param array<TKey, TValue> $items */
    final public function __construct(
        private $items = [],
    ) {}

    /**
     * @template TGetDefault
     *
     * @param  TKey  $key
     * @param  TGetDefault|(Closure(): TGetDefault)  $default
     * @return TValue|TGetDefault
     */
    public function get($key, $default = null): mixed
    {
        return $this->items[$key] ?? $this->resolveDefault($default);
    }

    /**
     * @template TGetDefault
     *
     * @param  TKey  $key
     * @param  TGetDefault|(Closure(): TGetDefault) $default
     * @return TValue|TGetDefault
     */
    public function getOrFalse(
        $key,
        $default = /** @return false */
        function (): bool {
            return false;
        },
    ): mixed {
        return $this->items[$key] ?? $this->resolveDefault($default);
    }

    /**
     * @template TGetDefault
     * @param TGetDefault|(Closure(): TGetDefault) $default
     * @return TGetDefault
     */
    private function resolveDefault(mixed $default): mixed
    {
        if ($default instanceof Closure) {
            /** @var TGetDefault */
            return $default();
        }

        return $default;
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
use_string($foo->get('a', 'default'));
use_string($foo->get('b', 'default'));
use_string($foo->get('c', 'default'));
use_string($foo->get('a', fn() => 'default'));
use_string($foo->get('b', fn() => 'default'));
use_string($foo->get('c', fn() => 'default'));
use_string($foo->getOrFalse('a') ?: 'default');
use_string($foo->getOrFalse('b') ?: 'default');
use_string($foo->getOrFalse('c') ?: 'default');
use_string($foo->getOrFalse('a', 'default'));
use_string($foo->getOrFalse('b', 'default'));
use_string($foo->getOrFalse('c', 'default'));
use_string($foo->getOrFalse('a', fn() => 'default'));
use_string($foo->getOrFalse('b', fn() => 'default'));
use_string($foo->getOrFalse('c', fn() => 'default'));

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
use_integer($bar->get('key_a', 0));
use_integer($bar->get('key_b', 0));
use_integer($bar->get('key_c', 0));
use_integer($bar->get('key_a', fn() => 0));
use_integer($bar->get('key_b', fn() => 0));
use_integer($bar->get('key_c', fn() => 0));
use_integer($bar->getOrFalse('key_a') ?: 0);
use_integer($bar->getOrFalse('key_b') ?: 0);
use_integer($bar->getOrFalse('key_c') ?: 0);
use_integer($bar->getOrFalse('key_a', 0));
use_integer($bar->getOrFalse('key_b', 0));
use_integer($bar->getOrFalse('key_c', 0));
use_integer($bar->getOrFalse('key_a', fn() => 0));
use_integer($bar->getOrFalse('key_b', fn() => 0));
use_integer($bar->getOrFalse('key_c', fn() => 0));
