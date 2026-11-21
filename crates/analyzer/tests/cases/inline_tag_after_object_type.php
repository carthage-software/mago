<?php

declare(strict_types=1);

/**
 * Compares two objects.
 *
 * @template TSource of object
 * @template TTarget of object
 *
 * {@see InlineTagAfterObjectTypeComparator}
 */
interface InlineTagAfterObjectType
{
    /**
     * @param TSource      $source
     * @param TTarget|null $target
     */
    public function __invoke(object $source, ?object $target): bool;
}

/**
 * @implements InlineTagAfterObjectType<stdClass, stdClass>
 */
final class InlineTagAfterObjectTypeComparator implements InlineTagAfterObjectType
{
    public function __invoke(object $source, ?object $target): bool
    {
        return $target !== null && $source == $target;
    }
}
