<?php

final class DynamicUnsetIndexCallback
{
    /** @throws RuntimeException */
    public static function stored(): void
    {
        throw new RuntimeException();
    }
}

/** @return 'other' */
function unknown_unset_index(): string
{
    return 'other';
}

function unset_dynamic_index_throw_preserves_callable_storage(): void
{
    $callbacks = [
        'job' => DynamicUnsetIndexCallback::stored(...),
        'other' => null,
    ];

    try {
        unset($callbacks[unknown_unset_index()]);
    } catch (Throwable) {
    }

    $callbacks['job']();
}
