<?php

class NotAString
{
}

/**
 * The effective return type here is `array{name: string, ...}`.
 * It might contain an 'email' key of any type.
 *
 * @return array{name: string, ...}
 */
function x(): array
{
    return ['name' => 'string', 'email' => new NotAString()];
}

/**
 * @return array{name: string, email?: string, ...}
 */
function y(): array
{
    // This return should be invalid.
    //
    // The type `array{name: string}` from x() is not a subtype of
    // `array{name: string, email?: string}` because x() could return an
    // array with an 'email' key of an incompatible type.
    //
    // @mago-expect analysis:less-specific-nested-return-statement
    return x();
}
