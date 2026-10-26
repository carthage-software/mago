<?php

final class DeadArrayUnionFailure extends Exception
{
}

final class DeadArrayUnionCallback
{
    /** @throws DeadArrayUnionFailure */
    public static function stored(): void
    {
        throw new DeadArrayUnionFailure();
    }
}

final class ConditionalArrayUnionFailure extends Exception
{
}

final class ConditionalArrayUnionCallback
{
    /** @throws ConditionalArrayUnionFailure */
    public static function stored(): void
    {
        throw new ConditionalArrayUnionFailure();
    }
}

function array_union_in_definitely_dead_code_does_not_store_callback(): void
{
    $callbacks = [];
    if (false) {
        $callbacks += [DeadArrayUnionCallback::stored(...)];
    }
}

function array_union_in_maybe_reachable_code_still_stores_callback(bool $condition): void
{
    $callbacks = [];
    if ($condition) {
        $callbacks += [ConditionalArrayUnionCallback::stored(...)];
    }
}
