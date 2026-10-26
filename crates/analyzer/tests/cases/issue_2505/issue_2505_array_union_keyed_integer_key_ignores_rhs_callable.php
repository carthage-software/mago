<?php

final class KeyedIntegerDuplicateCallbackFailure extends Exception
{
}

final class KeyedIntegerDuplicateCallback
{
    /** @throws KeyedIntegerDuplicateCallbackFailure */
    public static function stored(): void
    {
        throw new KeyedIntegerDuplicateCallbackFailure();
    }
}

function keyed_array_union_keeps_existing_integer_callback(): void
{
    $callbacks = [0 => static function (): void {}, 'named' => static function (): void {}];
    $callbacks += [KeyedIntegerDuplicateCallback::stored(...)];
}

keyed_array_union_keeps_existing_integer_callback();
