<?php

final class IntegerKeyDuplicateCallbackFailure extends Exception
{
}

final class IntegerKeyDuplicateCallback
{
    /** @throws IntegerKeyDuplicateCallbackFailure */
    public static function stored(): void
    {
        throw new IntegerKeyDuplicateCallbackFailure();
    }
}

function list_array_union_keeps_existing_callback(): void
{
    $callbacks = [static function (): void {}];
    $callbacks += [IntegerKeyDuplicateCallback::stored(...)];
}

list_array_union_keeps_existing_callback();
