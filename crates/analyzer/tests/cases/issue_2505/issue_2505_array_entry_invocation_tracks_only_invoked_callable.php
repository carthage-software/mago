<?php

final class InvokedArrayEntryFailure extends Exception
{
}

final class DeferredArrayEntryFailure extends Exception
{
}

final class InvokedArrayEntryCallback
{
    /** @throws InvokedArrayEntryFailure */
    public static function run(): void
    {
        throw new InvokedArrayEntryFailure();
    }
}

final class DeferredArrayEntryCallback
{
    /** @throws DeferredArrayEntryFailure */
    public static function run(): void
    {
        throw new DeferredArrayEntryFailure();
    }
}

function invoking_one_array_entry_keeps_sibling_callable_deferred(): void
{
    $callbacks = [
        'invoked' => InvokedArrayEntryCallback::run(...),
        'deferred' => DeferredArrayEntryCallback::run(...),
    ];

    try {
        $callbacks['invoked']();
    } catch (InvokedArrayEntryFailure) {
    }
}
