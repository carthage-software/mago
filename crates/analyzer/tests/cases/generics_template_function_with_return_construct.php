<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenWrapVal
{
    /** @param T $value */
    public function __construct(public mixed $value)
    {
    }
}

/**
 * @template T
 *
 * @param T $value
 *
 * @return GenWrapVal<T>
 */
function gen_wrap(mixed $value): GenWrapVal
{
    return new GenWrapVal($value);
}

/**
 * @param GenWrapVal<int> $w
 */
function take_wrap(GenWrapVal $w): int
{
    return $w->value;
}

/** @var int $value */
$value = 7;
take_wrap(gen_wrap($value));
