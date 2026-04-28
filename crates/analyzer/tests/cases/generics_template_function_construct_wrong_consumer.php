<?php

declare(strict_types=1);

/**
 * @template T
 */
final class GenWrapVal2
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
 * @return GenWrapVal2<T>
 */
function gen_wrap2(mixed $value): GenWrapVal2
{
    return new GenWrapVal2($value);
}

/**
 * @param GenWrapVal2<int> $w
 */
function take_wrap2(GenWrapVal2 $w): void
{
}

/** @mago-expect analysis:invalid-argument */
take_wrap2(gen_wrap2('hello'));
