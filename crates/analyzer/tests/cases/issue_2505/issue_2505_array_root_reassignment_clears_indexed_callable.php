<?php

final class ArrayRootResetFailure extends Exception
{
}

final class ArrayRootResetCallback
{
    /** @throws ArrayRootResetFailure */
    public static function stored(): void
    {
        throw new ArrayRootResetFailure();
    }
}

function resetting_array_root_clears_indexed_callback(): void
{
    $callbacks = [];
    $callbacks['job'] = ArrayRootResetCallback::stored(...);
    $callbacks = [];
}
