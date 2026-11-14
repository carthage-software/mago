<?php

declare(strict_types=1);

interface Animal {}

/** @template TModel of Animal */
final class Zoo
{
    /** @param TModel $first */
    public function __construct(public Animal $first) {}

    /**
     * @template TVal
     * @param TVal $value
     * @return (TVal is Animal ? Zoo<TVal> : string)
     */
    public function map(mixed $value): mixed
    {
        exit(1);
    }

    /**
     * @template TVal
     * @param TVal $value
     * @return (TVal is not Animal ? string : Zoo<TVal>)
     */
    public function mapNot(mixed $value): mixed
    {
        exit(1);
    }
}

/**
 * @template TVal
 * @param TVal $value
 * @return (TVal is Animal ? Zoo<TVal> : null)
 */
function wrap(mixed $value): ?Zoo
{
    exit(1);
}
