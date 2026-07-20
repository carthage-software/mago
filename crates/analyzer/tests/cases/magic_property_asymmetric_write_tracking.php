<?php

declare(strict_types=1);

namespace MagicPropertyAsymmetricWriteTracking;

/**
 * A wider `@property-write` than `@property-read` documents a `__set()` that converts: any
 * `string|int` may be written, but a read always yields `int|false`.
 *
 * @property-read int|false $converted
 * @property-write int|string $converted
 */
class Converter
{
    public function __get(string $_name): mixed
    {
        return 0;
    }

    public function __set(string $_name, mixed $_value): void
    {
    }
}

function takesInt(int $_value): void
{
}

$converter = new Converter();

// An `int` write is assignable to the read type, so a read tracks it: the `false` branch is
// excluded and the value passes where a plain `int` is required.
$converter->converted = 5;
takesInt($converter->converted);

// A `string` write shares nothing with the read type — `__set()` converts it — so the read
// falls back to the full `@property-read` type, whose `false` is not a valid `int` argument.
$converter->converted = 'today';
takesInt($converter->converted); // @mago-expect analysis:possibly-false-argument
