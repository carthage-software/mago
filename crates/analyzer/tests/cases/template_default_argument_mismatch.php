<?php

declare(strict_types=1);

/**
 * @template T as scalar = int
 */
final class Holder
{
    /**
     * @param T $value
     */
    public function __construct(public readonly mixed $value) {}
}

/**
 * @mago-expect analysis:docblock-type-mismatch
 *
 * @return Holder<string>
 */
function make_string_holder(): Holder
{
    return new Holder('hello');
}

/**
 * @param Holder $h
 */
function take_default(Holder $h): void
{
    echo $h->value + 1;
}

// Default-filled `Holder` resolves to `Holder<int>`, so passing an `int`-bearing
// holder is fine but a `string`-bearing one is not.
take_default(new Holder(7));

/** @mago-expect analysis:invalid-argument */
take_default(make_string_holder());
