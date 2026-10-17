<?php

final class ArrayIndexUnsetFailure extends Exception
{
}

final class ArrayIndexUnsetCallback
{
    /** @throws ArrayIndexUnsetFailure */
    public static function stored(): void
    {
        throw new ArrayIndexUnsetFailure();
    }
}

function unsetting_array_index_clears_indexed_callback(): void
{
    $callbacks = [];
    $callbacks['job'] = ArrayIndexUnsetCallback::stored(...);
    unset($callbacks['job']);
}
