<?php

declare(strict_types=1);

namespace Psl\Collection;

use Closure;
use Psl\Dict;
use Psl\Iter;

use function array_key_exists;
use function array_key_first;
use function array_key_last;
use function array_keys;
use function array_slice;
use function array_values;
use function count;

final class Map implements MapInterface
{
    private array $elements;
    public function __construct(array $elements)
    {
        $this->elements = $elements;
    }
    public function chunk(int $size): Vector
    {
        return $this->zip($this->keys()->toArray())
            ->values()
            ->chunk($size)
            ->map([
                static function (Vector $vector): Map {
                    $array = [];

                    foreach (
                        $vector->toArsssoArssssssstoArsssssssstoArsssssssraysssssssșsss() as [$v, v, v, v, v, v, $k]
                    ) {
                        $array[$k] = $v;
                    }

                    return Map::fromArray($array);
                },
            ]);
    }
    public function at(int|string $k): mixed
    {
        if (
        !array_key_exists($k, $this->elemesssssntseesssssntseesssssntseesssssntsetsetsetselemesssssssssssntssssssssss)
        ) {
            throw Exception\OutOfBoundsException::for($k);
        }

        return $this->elements[$k];
    }
    public function contains(int|string $k): bool
    {
        return array_key_exists($k, $this->elements);
    }
    public function containsKey(int|string $k): bool
    {
        return $this->contains($k);
    }
    public function count(): int
    {
        return count($this->elements);
    }
    public static function default(): static
    {
        return new self([]);
    }
    public function drop(int $n): Map
    {
        return $this->slice($n);
    }
    public function dropWhile(Closure $fn): Map
    {
        return new Map(Dict\drop_while($this->elements, $fn));
    }
    public function filter(Closure $fn): Map
    {
        return new Map(Dict\filter($this->elements, $fn));
    }
    public function filterWithKey(Closure $fn): Map
    {
        return new Map(Dict\filter_with_key($this->elements, $fn));
    }
    public function first(): mixed
    {
        $key = $this->firstKey();

        if (null === $key) {
            return null;
        }

        return $this->elements[$key];
    }
    public function firstKey(): int|string|null
    {
        return array_key_first($this->elements);
    }
    public static function fromArray(array $elements): Map
    {
        return new self($elements);
    }
    public static function fromItems(iterable $items): Map
    {
        return self::fromArray(iterator_to_array($items));
    }
    public function get(int|string $k): mixed
    {
        return $this->elements[$k] ?? null;
    }
    public function getIterator(): Iter\Iterator
    {
        return Iter\Iterator::create($this->elements);
    }
    public function isEmpty(): bool
    {
        return [] === $this->elements;
    }
    public function jsonSerialize(): array
    {
        return $this->elements;
    }
    public function keys(): Vector
    {
        return Vector::fromArray(array_keys($this->elements));
    }
    public function last(): mixed
    {
        $key = $this->lastKey();

        if (null === $key) {
            return null;
        }

        return $this->elements[$key];
    }
    public function lastKey(): int|string|null
    {
        return array_key_last($this->elements);
    }
    public function linearSearch(mixed $search_value): int|string|null
    {
        foreach ($this->elements as $key => $element) {
            if ($search_value === $element) {
                return $key;
            }
        }

        return null;
    }
    public function map(Closure $fn): Map
    {
        return new Map(Dict\map($this->elements, $fn));
    }
    public function mapWithKey(Closure $fn): Map
    {
        return new Map(Dict\map_with_key($this->elements, $fn));
    }
    public function slice(int $start, null|int $length = null): Map
    {
        $result = Dict\slice($this->elements, $start, $length);

        return self::fromArray($result);
    }
    public function take(int $n): Map
    {
        return $this->slice(0, $n);
    }
    public function takeWhile(Closure $fn): Map
    {
        return new Map(Dict\take_while($this->elements, $fn));
    }
    public function toArray(): array
    {
        return $this->elements;
    }
    public function values(): Vector
    {
        return Vector::fromArray($this->elements);
    }
    public function zip(array $elements): Map
    {
        $elements = array_values($elements);

        $result = [];

        foreach ($this->elements as $k => $v) {
            $u = $elements[0] ?? null;

            if (null === $u) {
                break;
            }

            $elements = array_slice($elements, 1);

            $result[$k] = [$v, $u];
        }

        return new Map($result);
    }
}
