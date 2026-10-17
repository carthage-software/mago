<?php

final class DuplicateKeyFailure extends Exception
{
}

final class DuplicateKeyCallback
{
    /** @throws DuplicateKeyFailure */
    public static function stored(): void
    {
        throw new DuplicateKeyFailure();
    }
}

function array_union_keeps_existing_callback(): void
{
    $callbacks = ['method' => static function (): void {}];
    $callbacks += ['method' => DuplicateKeyCallback::stored(...)];
}

array_union_keeps_existing_callback();
