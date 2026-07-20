<?php

declare(strict_types=1);

namespace PropertyHookGenericWriteTracking;

/**
 * The read type is the template `T`, resolved per instantiation; the `set` hook accepts the wider
 * `mixed`. A read goes through `get`, yielding the localized `T`, so a written value outside it
 * does not survive — the read type must be expanded and localized before it clamps the write.
 *
 * @template T
 */
class Box
{
    /** @var T */
    public mixed $value = null {
        get {
            return $this->value;
        }
        set(mixed $value) {
            $this->value = $value;
        }
    }
}

function takesInt(int $_value): void
{
}

/**
 * For `Box<int>` the read type localizes to `int`, so a `string` write is converted away on read:
 * the clamp falls back to `int` (a valid argument) rather than leaking the written `string`.
 *
 * @param Box<int> $box
 */
function f(Box $box): void
{
    $box->value = 'converted';
    takesInt($box->value);
}
