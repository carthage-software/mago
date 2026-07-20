<?php

declare(strict_types=1);

namespace PropertyHookAsymmetricWriteTracking;

/**
 * A `set` hook whose parameter type is wider than the property (read) type documents a hook
 * that converts: any `string|int|false` may be written, but a read always yields `int|false`
 * (the `get` hook returns the backing property, never the raw written value).
 */
class Converter
{
    public int|false $converted = 0 {
        get {
            return $this->converted;
        }
        set(string|int|false $value) {
            $this->converted = (int) $value;
        }
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

// A `string` write shares nothing with the read type — the `set` hook converts it — so the
// read falls back to the full read type, whose `false` is not a valid `int` argument.
$converter->converted = 'today';
takesInt($converter->converted); // @mago-expect analysis:possibly-false-argument
