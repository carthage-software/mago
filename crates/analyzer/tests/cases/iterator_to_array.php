<?php

/**
 * @template Tk of array-key
 * @template Tv
 */
final class Map
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
     * Creates and returns a default instance of {@see Map}.
     *
     * @return static A default instance of {@see Map}.
     *
     * @pure
     */
    public static function default(): static
    {
        return new self([]);
    }

    /**
     * @template Tsk of array-key
     * @template Tsv
     *
     * @param array<Tsk, Tsv> $elements
     *
     * @return Map<Tsk, Tsv>
     *
     * @pure
     */
    public static function fromArray(array $elements): Map
    {
        return new self($elements);
    }

    /**
     * @template Tsk of array-key
     * @template Tsv
     *
     * @param array<Tsk, Tsv> $items
     *
     * @return Map<Tsk, Tsv>
     */
    public static function fromItems(iterable $items): Map
    {
        return self::fromArray(iterator_to_array($items));
    }
}
