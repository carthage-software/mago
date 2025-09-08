<?php

/**
 * @template Tk
 * @template Tv
 * @template T
 *
 * @param iterable<Tk, Tv> $iterable
 * @param (Closure(Tv): T) $function
 *
 * @return ($iterable is non-empty-array ? non-empty-list<T> : list<T>)
 */
function vec_map(iterable $iterable, Closure $function): array
{
    return vec_map($iterable, $function);
}

/**
 * @template Tv
 * @template Tu
 *
 * @param iterable<Tv> $first
 * @param iterable<Tu> $second
 *
 * @return list<array{0: Tv, 1: Tu}>
 */
function vec_zip(iterable $first, iterable $second): array
{
    return vec_zip($first, $second);
}

/**
 * @template T
 *
 * @param iterable<T> $iterable
 * @param positive-int $size
 *
 * @return list<list<T>>
 */
function vec_chunk(iterable $iterable, int $size): array
{
    return vec_chunk($iterable, $size);
}

/**
 * @template Tk of array-key
 * @template Tv
 */
interface CollectionInterface
{
    /**
     * @return array<Tk, Tv>
     */
    public function toArray(): array;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return CollectionInterface<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): CollectionInterface;

    /**
     * @param positive-int $size
     *
     * @return CollectionInterface<int<0, max>, static<Tk, Tv>>
     */
    public function chunk(int $size): CollectionInterface;
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends CollectionInterface<Tk, Tv>
 */
interface MutableCollectionInterface extends CollectionInterface
{
    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MutableCollectionInterface<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): MutableCollectionInterface;

    /**
     * @param positive-int $size
     *
     * @return MutableCollectionInterface<int<0, max>, static<Tk, Tv>>
     */
    public function chunk(int $size): MutableCollectionInterface;
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends CollectionInterface<Tk, Tv>
 */
interface AccessibleCollectionInterface extends CollectionInterface
{
    /**
     * @return AccessibleCollectionInterface<int<0, max>, Tv>
     */
    public function values(): AccessibleCollectionInterface;

    /**
     * @return AccessibleCollectionInterface<int<0, max>, Tk>
     */
    public function keys(): AccessibleCollectionInterface;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return AccessibleCollectionInterface<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): AccessibleCollectionInterface;

    /**
     * @param positive-int $size
     *
     * @return AccessibleCollectionInterface<int<0, max>, static<Tk, Tv>>
     */
    public function chunk(int $size): AccessibleCollectionInterface;
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends AccessibleCollectionInterface<Tk, Tv>
 * @extends MutableCollectionInterface<Tk, Tv>
 */
interface MutableAccessibleCollectionInterface extends AccessibleCollectionInterface, MutableCollectionInterface
{
    /**
     * @return MutableAccessibleCollectionInterface<int<0, max>, Tv>
     */
    public function values(): MutableAccessibleCollectionInterface;

    /**
     * @return MutableAccessibleCollectionInterface<int<0, max>, Tk>
     */
    public function keys(): MutableAccessibleCollectionInterface;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MutableAccessibleCollectionInterface<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): MutableAccessibleCollectionInterface;

    /**
     * @param positive-int $size
     *
     * @return MutableAccessibleCollectionInterface<int<0, max>, static<Tk, Tv>>
     */
    public function chunk(int $size): MutableAccessibleCollectionInterface;
}

/**
 * @template T
 *
 * @extends AccessibleCollectionInterface<int<0, max>, T>
 */
interface VectorInterface extends AccessibleCollectionInterface
{
    /**
     * @return list<T>
     */
    public function toArray(): array;

    /**
     * @return VectorInterface<T>
     */
    public function values(): VectorInterface;

    /**
     * @return VectorInterface<int<0, max>>
     */
    public function keys(): VectorInterface;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return VectorInterface<array{0: T, 1: Tu}>
     */
    public function zip(array $elements): VectorInterface;

    /**
     * @param positive-int $size
     *
     * @return VectorInterface<static<T>>
     */
    public function chunk(int $size): VectorInterface;
}

/**
 * @template T
 *
 * @extends VectorInterface<T>
 * @extends MutableAccessibleCollectionInterface<int<0, max>, T>
 */
interface MutableVectorInterface extends MutableAccessibleCollectionInterface, VectorInterface
{
    /**
     * @return list<T>
     */
    public function toArray(): array;

    /**
     * @return MutableVectorInterface<T>
     */
    public function values(): MutableVectorInterface;

    /**
     * @return MutableVectorInterface<int<0, max>>
     */
    public function keys(): MutableVectorInterface;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MutableVectorInterface<array{0: T, 1: Tu}>
     */
    public function zip(array $elements): MutableVectorInterface;

    /**
     * @param positive-int $size
     *
     * @return MutableVectorInterface<static<T>>
     */
    public function chunk(int $size): MutableVectorInterface;
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends AccessibleCollectionInterface<Tk, Tv>
 */
interface MapInterface extends AccessibleCollectionInterface
{
    /**
     * @return VectorInterface<Tv>
     */
    public function values(): VectorInterface;

    /**
     * @return VectorInterface<Tk>
     */
    public function keys(): VectorInterface;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MapInterface<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): MapInterface;

    /**
     * @param positive-int $size
     *
     * @return VectorInterface<static<Tk, Tv>>
     */
    public function chunk(int $size): VectorInterface;
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @extends MapInterface<Tk, Tv>
 * @extends MutableAccessibleCollectionInterface<Tk, Tv>
 */
interface MutableMapInterface extends MapInterface, MutableAccessibleCollectionInterface
{
    /**
     * @return MutableVectorInterface<Tv>
     */
    public function values(): MutableVectorInterface;

    /**
     * @return MutableVectorInterface<Tk>
     */
    public function keys(): MutableVectorInterface;

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MutableMapInterface<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): MutableMapInterface;

    /**
     * @param positive-int $size
     *
     * @return MutableVectorInterface<static<Tk, Tv>>
     */
    public function chunk(int $size): MutableVectorInterface;
}

/**
 * @template T
 *
 * @implements MutableVectorInterface<T>
 */
final class MutableVector implements MutableVectorInterface
{
    /**
     * @var list<T> $elements
     */
    private array $elements = [];

    /**
     * @param array<array-key, T> $elements
     */
    public function __construct(array $elements)
    {
        foreach ($elements as $element) {
            $this->elements[] = $element;
        }
    }

    /**
     * @template Ts
     *
     * @param array<array-key, Ts> $elements
     *
     * @return MutableVector<Ts>
     */
    public static function fromArray(array $elements): MutableVector
    {
        return new self($elements);
    }

    /**
     * @return list<T>
     */
    public function toArray(): array
    {
        return $this->elements;
    }

    /**
     * @return MutableVector<T>
     */
    public function values(): MutableVector
    {
        return MutableVector::fromArray($this->elements);
    }

    /**
     * @return MutableVector<int<0, max>>
     */
    public function keys(): MutableVector
    {
        return MutableVector::fromArray(array_keys($this->elements));
    }

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MutableVector<array{0: T, 1: Tu}>
     */
    public function zip(array $elements): MutableVector
    {
        return MutableVector::fromArray(vec_zip($this->elements, $elements));
    }

    /**
     * @param positive-int $size
     *
     * @return MutableVector<MutableVector<T>>
     */
    public function chunk(int $size): MutableVector
    {
        return static::fromArray(vec_map(
            vec_chunk($this->toArray(), $size),
            /**
             * @param list<T> $chunk
             *
             * @return MutableVector<T>
             */
            static fn(array $chunk): MutableVector => MutableVector::fromArray($chunk),
        ));
    }
}

/**
 * @template Tk of array-key
 * @template Tv
 *
 * @implements MutableMapInterface<Tk, Tv>
 */
final class MutableMap implements MutableMapInterface
{
    /**
     * @var array<Tk, Tv> $elements
     */
    private array $elements;

    /**
     * @param array<Tk, Tv> $elements
     */
    public function __construct(array $elements)
    {
        $this->elements = $elements;
    }

    /**
     * @template Tsk of array-key
     * @template Tsv
     *
     * @param array<Tsk, Tsv> $elements
     *
     * @return MutableMap<Tsk, Tsv>
     */
    public static function fromArray(array $elements): MutableMap
    {
        return new self($elements);
    }

    /**
     * @return array<Tk, Tv>
     */
    public function toArray(): array
    {
        return $this->elements;
    }

    /**
     * @return MutableVector<Tv>
     */
    public function values(): MutableVector
    {
        return MutableVector::fromArray($this->elements);
    }

    /**
     * @return MutableVector<Tk>
     */
    public function keys(): MutableVector
    {
        return MutableVector::fromArray(array_keys($this->elements));
    }

    /**
     * @template Tu
     *
     * @param array<array-key, Tu> $elements
     *
     * @return MutableMap<Tk, array{0: Tv, 1: Tu}>
     */
    public function zip(array $elements): MutableMap
    {
        return $this->zip($elements); // stub
    }

    /**
     * @param positive-int $size
     *
     * @return MutableVector<MutableMap<Tk, Tv>>
     */
    public function chunk(int $size): MutableVector
    {
        $chunks = $this->zip($this->keys()->toArray())
            ->values()
            ->chunk($size)
            ->toArray();

        return MutableVector::fromArray(vec_map(
            $chunks,
            /**
             * @param MutableVector<array{0: Tv, 1: Tk}> $vector
             *
             * @return MutableMap<Tk, Tv>
             */
            static function (MutableVector $vector): MutableMap {
                /** @var array<Tk, Tv> $array */
                $array = [];
                foreach ($vector->toArray() as [$v, $k]) {
                    $array[$k] = $v;
                }

                return MutableMap::fromArray($array);
            },
        ));
    }
}
