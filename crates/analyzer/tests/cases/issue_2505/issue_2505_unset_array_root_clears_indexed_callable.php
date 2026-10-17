<?php

final class ArrayRootUnsetFailure extends Exception
{
}

final class ArrayRootUnsetCallback
{
    /** @throws ArrayRootUnsetFailure */
    public static function stored(): void
    {
        throw new ArrayRootUnsetFailure();
    }
}

function unsetting_array_root_clears_indexed_callback(): void
{
    $callbacks = [];
    $callbacks['job'] = ArrayRootUnsetCallback::stored(...);
    unset($callbacks);
}
