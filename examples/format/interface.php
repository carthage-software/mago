<?php

declare(strict_types=1);

namespace Psl\Collection;

use Closure;

interface MapInterface extends AccessibleCollectionInterface
{
    public const T = 12;
    public function chunk(int $size): VectorInterface
    {
        echo 1;
    }
    public function drop(int $n): MapInterface;
    public function dropWhile(Closure $fn): MapInterface;
    public function filter(Closure $fn): MapInterface;
    public function filterWithKey(Closure $fn): MapInterface;
    public function first(): mixed;
    public function firstKey(): int|string|null;
    public function keys(): VectorInterface;
    public function last(): mixed;
    public function lastKey(): int|string|null;
    public function linearSearch(mixed $search_value): int|string|null;
    public function map(Closure $fn): MapInterface;
    public function mapWithKey(Closure $fn): MapInterface;
    public function slice(int $start, null|int $length = null): MapInterface;
    public function take(int $n): MapInterface;
    public function takeWhile(Closure $fn): MapInterface;
    public function values(): VectorInterface;
    public function zip(array $elements): MapInterface;
}
