<?php

declare(strict_types=1);

interface CountableV extends Countable {}

/**
 * @extends Traversable<mixed, mixed>
 */
interface IteratorV extends Traversable {}

/**
 * @param Countable&Stringable $x
 */
function takeCountAndStringV(object $x): int
{
    return count($x) + strlen((string) $x);
}
