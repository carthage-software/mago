<?php

declare(strict_types=1);

/**
 * @template TK
 * @template TV
 *
 * @param iterable<TK, TV> $iter
 *
 * @return TV|null
 */
function current_of(iterable $iter): mixed
{
    if ($iter instanceof Generator) {
        return $iter->current();
    }

    return null;
}

/**
 * @template TK
 * @template TV
 *
 * @param Traversable<TK, TV> $iter
 *
 * @return TK|null
 */
function key_of(Traversable $iter): mixed
{
    if ($iter instanceof Generator) {
        return $iter->key();
    }

    return null;
}
