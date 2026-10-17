<?php

final class GenericListCallbackFailure extends Exception
{
}

final class GenericListCallback
{
    /** @throws GenericListCallbackFailure */
    public static function stored(): void
    {
        throw new GenericListCallbackFailure();
    }
}

/** @param list<callable(): void> $callbacks */
function add_to_generic_callback_list(array $callbacks): void
{
    $callbacks += [GenericListCallback::stored(...)];
}
