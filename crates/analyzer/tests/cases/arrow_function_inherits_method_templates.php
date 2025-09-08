<?php

namespace Psl\Iter;

use Closure;
use Generator;

/**
 * @template Tk
 * @template Tv
 */
final class Iterator
{
    /**
     * @var null|Generator<Tk, Tv, mixed, mixed>
     */
    public null|Generator $generator;

    /**
     * @var array<int, array{0: Tk, 1: Tv}>
     */
    public array $entries = [];

    /**
     *  Whether the current value/key pair has been added to the local entries.
     */
    public bool $saved = true;

    /**
     * Current cursor position for the local entries.
     */
    public int $position = 0;

    /**
     * The size of the generator.
     *
     * @var null|int<0, max>
     */
    public null|int $count = null;

    /**
     * @param Generator<Tk, Tv, mixed, mixed> $generator
     */
    public function __construct(Generator $generator)
    {
        $this->generator = $generator;
    }

    /**
     * Create an iterator from a factory.
     *
     * @template Tsk
     * @template Tsv
     *
     * @param (Closure(): iterable<Tsk, Tsv>) $factory
     *
     * @return Iterator<Tsk, Tsv>
     */
    public static function from(Closure $factory): Iterator
    {
        return self::create($factory());
    }

    /**
     * Create an iterator from an iterable.
     *
     * @template Tsk
     * @template Tsv
     *
     * @param iterable<Tsk, Tsv> $iterable
     *
     * @return Iterator<Tsk, Tsv>
     */
    public static function create(iterable $iterable): Iterator
    {
        if ($iterable instanceof Generator) {
            return new self($iterable);
        }

        $factory =
            /**
             * @return Generator<Tsk, Tsv, mixed, mixed>
             */
            static fn(): Generator => yield from $iterable;

        return new self($factory());
    }
}
