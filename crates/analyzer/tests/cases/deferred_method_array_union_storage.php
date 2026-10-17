<?php

final class MethodStorageFailure extends Exception
{
}

final class DeferredMethodStorage
{
    /**
     * @throws MethodStorageFailure
     */
    public static function stored(): void
    {
        throw new MethodStorageFailure();
    }
}

function store_method_in_array_union(): void
{
    $callbacks = ['existing' => static function (): void {}];
    $callbacks += ['method' => DeferredMethodStorage::stored(...)];
}

store_method_in_array_union();
