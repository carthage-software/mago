<?php

declare(strict_types=1);

interface CountableV extends Countable
{
}

interface IteratorV extends Traversable
{
}

/**
 * @param Countable&Stringable $x
 */
function takeCountAndStringV(object $x): int
{
    return count($x) + strlen((string) $x);
}
