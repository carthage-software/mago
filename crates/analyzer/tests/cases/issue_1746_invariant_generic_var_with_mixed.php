<?php

namespace Fixture;

/**
 * @template T
 */
interface Box
{
    /** @return T */
    public function get(): mixed;
}

/**
 * @return Box
 */
function box(Box $seed): Box
{
    return $seed;
}

/**
 * @param Box $seed
 */
function consume(Box $seed): void
{
    /** @var Box<resource> $b */
    $b = box($seed);

    $b->get();
}
